use anyhow::Result;
use imageproc::point::Point;
use ndarray::{Array2, Axis};

pub struct PostProcessor {
    pub threshold: f32,
}
const LABELS: [&str; 20] = [
    "paragraph_title",
    "image",
    "text",
    "number",
    "abstract",
    "content",
    "figure_title",
    "formula",
    "table",
    "reference",
    "doc_title",
    "footnote",
    "header",
    "algorithm",
    "footer",
    "seal",
    "chart",
    "formula_number",
    "aside_text",
    "reference_content",
];

impl Default for PostProcessor {
    fn default() -> Self {
        PostProcessor { threshold: 0.5 }
    }
}

#[derive(Debug)]
pub struct BoxResult {
    pub coordinate: [f32; 4],
    pub label: String,
    pub score: f32,
}

impl PostProcessor {
    pub fn process(&self, pred: &Array2<f32>) -> Result<Vec<BoxResult>> {
        let mut result = Vec::new();
        for (i, row) in pred.axis_iter(Axis(0)).enumerate() {
            let label_id = row[0] as usize;
            let label = LABELS[label_id].to_string();
            let score = row[1];
            if score < self.threshold {
                continue;
            }
            let coordinate = [row[2], row[3], row[4], row[5]];
            let res = BoxResult {
                coordinate,
                label,
                score,
            };
            result.push(res)
        }
        Ok(result)
    }
}

fn distance(p1: &Point<u32>, p2: &Point<u32>) -> f32 {
    let d = ((p1.x as f32 - p2.x as f32) * (p1.x as f32 - p2.x as f32)
        + (p1.y as f32 - p2.y as f32) * (p1.y as f32 - p2.y as f32))
        .sqrt();
    d
}
