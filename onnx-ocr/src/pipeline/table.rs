use anyhow::Result;
use image::{
    RgbImage,
    imageops::{crop_imm, rotate90, rotate180, rotate270},
};
use itertools::Itertools;

use crate::{
    doc_text_ori::predictor::RotateAngle,
    model_context::ModelContext,
    pipeline::ocr::{self},
    table_cell_detection::predictor::TableCelltResult,
    table_cls::predictor::TableType,
};

#[derive(Debug, Clone)]
pub struct TableCell {
    pub coordinate: [f32; 4],
    pub score: f32,
    pub content: String,
}

impl TableCell {
    pub fn new(coordinate: [f32; 4], score: f32, content: String) -> Self {
        TableCell {
            coordinate,
            score,
            content,
        }
    }
}

pub fn extract_table(context: &ModelContext, img: &RgbImage) -> Result<Table> {
    let doc_text_ori_predictor = &context.doc_text_ori_predictor;
    let doc_angle = doc_text_ori_predictor.predict_image(img)?;
    let pre_img = match doc_angle {
        RotateAngle::R0 => img.to_owned(),
        RotateAngle::R90 => rotate90(img),
        RotateAngle::R180 => rotate180(img),
        RotateAngle::R270 => rotate270(img),
    };

    let table_cls_predictor = &context.table_cls_predictor;
    let table_type = table_cls_predictor.predict_image(&pre_img)?;
    let table_cells_result = match table_type {
        TableType::Wired => {
            let predictor = &context.wired_table_cell_predictor;
            predictor.predict_image(img)?
        }
        TableType::Wireless => {
            let predictor = &context.wireless_table_cell_predictor;
            predictor.predict_image(img)?
        }
    };
    let table_cells_result = cells_det_result_nms(table_cells_result)?;

    let table_structure = match table_type {
        TableType::Wired => {
            let predictor = &context.wired_table_structure_predictor;
            predictor.predict_image(img)?
        }
        TableType::Wireless => {
            let predictor = &context.wireless_table_structure_predictor;
            predictor.predict_image(img)?
        }
    };
    let mut table_cells = Vec::new();
    for cell in table_cells_result.iter() {
        let [x1, y1, x2, y2] = cell.coordinate;
        let width = (x2 - x1).ceil() as u32;
        let height = (y2 - y1).ceil() as u32;
        let cell_img = crop_imm(img, x1.ceil() as u32, y1.ceil() as u32, width, height).to_image();
        let cell_ocr_reslt = ocr::ocr(context, &cell_img)?;
        let content = cell_ocr_reslt.iter().map(|v| &v.content).join("");
        let table_cell = TableCell::new(cell.coordinate, cell.score, content);
        table_cells.push(table_cell);
    }
    table_cells.sort_by(|a, b| a.coordinate[1].total_cmp(&b.coordinate[1]));
    let mut rows = Vec::new();
    let mut current_row = Vec::new();
    let mut current_y = 0.0;
    let mut start_row = true;
    for tcell in table_cells {
        let [x1, y1, x2, y2] = tcell.coordinate;
        if start_row {
            current_row.push(tcell);
            current_y = y1;
            start_row = false;
        } else {
            if (y1 - current_y).abs() < 10.0 {
                current_row.push(tcell);
            } else {
                current_row.sort_by(|a, b| a.coordinate[0].total_cmp(&b.coordinate[0]));
                rows.push(current_row.to_owned());
                current_row.clear();
                current_row.push(tcell);
                current_y = y1;
            }
        }
    }
    if !current_row.is_empty() {
        current_row.sort_by(|a, b| a.coordinate[0].total_cmp(&b.coordinate[0]));
        rows.push(current_row.to_owned());
    }
    let table = Table::new(rows, table_structure.table_labels);
    Ok(table)
}

#[derive(Debug)]
pub struct Table {
    rows: Vec<Vec<TableCell>>,
    structure: Vec<String>,
}

impl Table {
    pub fn new(rows: Vec<Vec<TableCell>>, structure: Vec<String>) -> Self {
        Table { rows, structure }
    }

    pub fn rows(&self) -> &[Vec<TableCell>] {
        self.rows.as_slice()
    }

    pub fn to_html(&self) -> String {
        // TODO if structure not match cells results
        let mut html_tags: Vec<String> = Vec::new();
        let mut cell_index = 0;
        let mut row_index = 0;
        for tag in self.structure.iter() {
            match tag.as_str() {
                "<td></td>" => {
                    let cell = &self.rows[row_index][cell_index];
                    html_tags.push(format!("<td>{}</td>", cell.content));
                    cell_index += 1;
                }
                "</td>" => {
                    let cell = &self.rows[row_index][cell_index];
                    html_tags.push(cell.content.clone());
                    html_tags.push("</td>".to_string());
                    cell_index += 1;
                }
                "</tr>" => {
                    cell_index = 0;
                    row_index += 1;
                }
                _ => {
                    html_tags.push(tag.to_owned());
                }
            }
        }
        html_tags.join("")
    }
}

fn cells_det_result_nms(cells_det_results: Vec<TableCelltResult>) -> Result<Vec<TableCelltResult>> {
    let mut sorted_results = cells_det_results;
    sorted_results.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut nms_result = Vec::new();
    let iou_threshold = 0.3;
    while !sorted_results.is_empty() {
        let first = sorted_results.pop().unwrap();
        sorted_results.retain(|result| {
            let iou = compute_iou(&first.coordinate, &result.coordinate);
            iou <= iou_threshold
        });
        nms_result.push(first);
    }
    return Ok(nms_result);
}

fn compute_iou(box1: &[f32; 4], box2: &[f32; 4]) -> f32 {
    let x_left = box1[0].max(box2[0]);
    let y_top = box1[1].max(box2[1]);
    let x_right = box1[2].min(box2[2]);
    let y_bottom = box1[3].min(box2[3]);
    if x_left > x_right || y_bottom < y_top {
        return 0.0;
    }
    let intersection_area = (x_right - x_left) * (y_bottom - y_top);
    let area_box1 = (box1[2] - box1[0]) * (box1[3] - box1[1]);
    let area_box2 = (box2[2] - box2[0]) * (box2[3] - box2[1]);
    let iou = intersection_area / (area_box1 + area_box2 - intersection_area);
    iou
}
