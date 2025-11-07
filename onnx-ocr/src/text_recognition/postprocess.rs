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
        let mut idxs = Vec::new();
        let mut scores = Vec::new();

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
            idxs.push(max_index);
            scores.push(max_score);
        }
        let num_char = idxs.len();
        // remove duplicated
        for i in 0..num_char - 1 {
            let j = i + 1;
            if i == 0 {
                result.push((idxs[i], scores[i]));
                continue;
            }
            if idxs[j] == idxs[i] {
                continue;
            }
            result.push((idxs[j], scores[j]));
        }
        Ok(result)
    }
}
