use anyhow::Result;
use image::{
    Rgb, Rgb32FImage, RgbImage,
    imageops::{FilterType, resize},
};
use ndarray::Array4;

use crate::common::imgproc::convert_rgb_to_rgb32f;

pub struct PreProcessor {}

pub struct PreOutput {
    pub ori_img: RgbImage,
    pub ratio_h: f32,
    pub ratio_w: f32,
    pub input_width: u32,
    pub input_height: u32,
    pub input: Rgb32FImage,
}
impl PreOutput {
    pub fn get_input_as_ndarray(&self) -> Array4<f32> {
        let mut input =
            Array4::<f32>::zeros((1, 3, self.input_height as usize, self.input_width as usize));

        for y in 0..self.input_height as usize {
            for x in 0..self.input_width as usize {
                let pixel = self.input.get_pixel(x as u32, y as u32);
                let [r, g, b] = pixel.0;
                input[[0, 0, y, x]] = r;
                input[[0, 1, y, x]] = g;
                input[[0, 2, y, x]] = b;
            }
        }
        return input;
    }
}

impl Default for PreProcessor {
    fn default() -> Self {
        PreProcessor {}
    }
}

impl PreProcessor {
    pub fn process(&self, img: &RgbImage) -> Result<PreOutput> {
        let result = self.resize(img)?;
        Ok(result)
    }

    fn resize(&self, img: &RgbImage) -> Result<PreOutput> {
        let h = img.height();
        let w = img.width();
        let rh = 800;
        let rw = 800;
        let rimg = resize(img, rw, rh, FilterType::Triangle);
        let result = PreOutput {
            ori_img: img.to_owned(),
            input_width: rw,
            input_height: rh,
            ratio_w: rw as f32 / w as f32,
            ratio_h: rh as f32 / h as f32,
            input: convert_rgb_to_rgb32f(&rimg),
        };
        return Ok(result);
    }
}
