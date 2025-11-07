use image::{Rgb32FImage, RgbImage};

pub struct PreProcessor {}

impl Default for PreProcessor {
    fn default() -> Self {
        PreProcessor {}
    }
}

impl PreProcessor {
    pub fn process(&self, img: &RgbImage) -> Rgb32FImage {
        unimplemented!()
    }
}
