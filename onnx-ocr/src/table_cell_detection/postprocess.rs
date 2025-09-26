use anyhow::Result;
use ndarray::{Array2, Axis};

pub struct PostProcessor {
    pub threshold: f32,
}

impl Default for PostProcessor {
    fn default() -> Self {
        PostProcessor { threshold: 0.3 }
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
            let label = "cell".to_string();
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
