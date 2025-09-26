use anyhow::Result;
use ndarray::Array2;

pub struct PostProcessor {}

impl Default for PostProcessor {
    fn default() -> Self {
        PostProcessor {}
    }
}

impl PostProcessor {
    pub fn process(&self, pred: &Array2<f32>) -> Result<Vec<(u32, f32)>> {
        let (h, w) = pred.dim();
        let mut result = Vec::new();
        for i in 0..h {
            let mut max_index = 0;
            let mut max_score = 0.0;
            for j in 0..w {
                let s = pred.get((i, j)).unwrap_or(&0.0).to_owned();
                if s > max_score {
                    max_score = s;
                    max_index = j as u32;
                }
            }
            result.push((max_index, max_score));
        }
        Ok(result)
    }
}
