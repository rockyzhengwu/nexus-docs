use anyhow::Result;
use ndarray::{Array2, Axis};

pub struct PostProcessor {
    character: Vec<String>,
}

#[derive(Debug)]
pub struct TableStructure {
    pub bboxs: Vec<Vec<u32>>,
    pub table_labels: Vec<String>,
}

impl PostProcessor {
    pub fn new(character: Vec<String>) -> Self {
        PostProcessor { character }
    }

    pub fn process(
        &self,
        bbox_pred: &Array2<f32>,
        structure_pred: &Array2<f32>,
        ori_shape: &[u32; 2],
    ) -> Result<TableStructure> {
        let [ori_w, ori_h] = ori_shape;

        let ratio_w = 512.0 / ori_w.to_owned() as f32;
        let ratio_h = 512.0 / ori_h.to_owned() as f32;
        let ratio = ratio_w.min(ratio_h);
        let scale = 512.0 / ratio;
        let mut bbox_list = Vec::new();
        let mut labels = Vec::new();

        for (row_idx, row) in structure_pred.axis_iter(Axis(0)).enumerate() {
            let mut max_index = 0;
            let mut max_value = row[0];
            for (idx, v) in row.iter().enumerate() {
                if *v > max_value {
                    max_value = *v;
                    max_index = idx;
                }
            }
            if max_index == 49 {
                break;
            }
            if max_index == 0 {
                continue;
            }
            let bbox = bbox_pred.row(row_idx).to_vec();
            let bbox: Vec<u32> = bbox
                .iter()
                .map(|v| (v.to_owned() * scale).round() as u32)
                .collect();
            let label = self.character.get(max_index);
            match label {
                Some(ls) => {
                    labels.push(ls.to_owned());
                    if ls == "td" || ls == "</td>" || ls == "<td></td>" {
                        bbox_list.push(bbox);
                    }
                }
                None => {
                    println!("invalid label index");
                    continue;
                }
            }
        }
        let structure = TableStructure {
            bboxs: bbox_list,
            table_labels: labels,
        };
        Ok(structure)
    }
}
