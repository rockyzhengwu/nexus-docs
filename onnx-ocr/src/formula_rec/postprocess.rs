use anyhow::Result;
use ndarray::Array2;

pub struct PostProcessor {}

impl Default for PostProcessor {
    fn default() -> Self {
        Self {}
    }
}

impl PostProcessor {
    pub fn process(&self, pred: &Array2<f32>) -> Result<Vec<(u32, f32)>> {
        unimplemented!()
    }
}
