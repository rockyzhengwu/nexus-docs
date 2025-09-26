use anyhow::Result;
use image::{
    Rgb, Rgb32FImage, RgbImage,
    imageops::{FilterType, resize},
};
use ndarray::Array4;

use crate::common::imgproc::convert_rgb_to_rgb32f;

pub struct PreProcessor {
    target_size: u32,
}

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
        let target_size = 640;

        PreProcessor { target_size }
    }
}

impl PreProcessor {
    pub fn process(&self, img: &RgbImage) -> Result<PreOutput> {
        let mut result = self.resize(img)?;
        let nimg = self.normalize(&result.input);
        result.input = nimg;
        Ok(result)
    }

    fn normalize(&self, img: &Rgb32FImage) -> Rgb32FImage {
        let w = img.width();
        let h = img.height();
        let mut nimg = Rgb32FImage::new(w, h);
        for x in 0..w {
            for y in 0..h {
                let pixel = img.get_pixel(x, y);
                let [r, g, b] = pixel.0;
                nimg.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        return nimg;
    }

    fn resize(&self, img: &RgbImage) -> Result<PreOutput> {
        let h = img.height();
        let w = img.width();
        let rh = self.target_size;
        let rw = self.target_size;
        if rw == w && rh == h {
            let result = PreOutput {
                ori_img: img.to_owned(),
                input_width: rw,
                input_height: rh,
                ratio_h: 1.0,
                ratio_w: 1.0,
                input: convert_rgb_to_rgb32f(img),
            };
            return Ok(result);
        }
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
