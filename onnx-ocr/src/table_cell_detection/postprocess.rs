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
    pub fn process(&self, pred: &Array2<f32>, w: f32, h: f32) -> Result<Vec<BoxResult>> {
        let mut result = Vec::new();
        for (i, row) in pred.axis_iter(Axis(0)).enumerate() {
            let label = "cell".to_string();
            let score = row[1];
            if score < self.threshold {
                continue;
            }
            let minx = row[2].max(0.0);
            let miny = row[3].max(0.0);
            let maxx = row[4].min(w);
            let maxy = row[5].min(h);
            if minx > maxx || maxy < miny {
                continue;
            }
            let coordinate = [minx, miny, maxx, maxy];
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
