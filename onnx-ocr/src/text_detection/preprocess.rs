use anyhow::Result;
use image::{
    Rgb, Rgb32FImage, RgbImage,
    imageops::{FilterType, resize},
};
use ndarray::Array4;

use crate::common::imgproc::convert_rgb_to_rgb32f;

pub struct PreProcessor {
    max_side_limit: usize,
    limit_side_len: usize,
    mean: [f32; 3],
    std: [f32; 3],
    alpha: [f32; 3],
    beta: [f32; 3],
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
        let mean = [0.485, 0.456, 0.406];
        let std = [0.229, 0.224, 0.225];
        let alpha = [1.0 / 0.229, 1.0 / 0.224, 1.0 / 0.225];
        let beta = [-0.485 / 0.229, -0.456 / 0.224, -0.406 / 0.225];

        println!("beta alpha:{:?},{:?}", alpha, beta);
        PreProcessor {
            max_side_limit: 4000,
            limit_side_len: 960,
            mean,
            std,
            alpha,
            beta,
        }
    }
}

impl PreProcessor {
    pub fn process(&self, img: RgbImage) -> Result<PreOutput> {
        let w = img.width();
        let h = img.height();
        let paded_image = if (w + h) < 64 as u32 {
            self.pad_image(&img)
        } else {
            img
        };
        let mut result = self.resize(&paded_image)?;
        let nimg = self.normalize(&result.input);
        result.input = nimg;
        Ok(result)
    }
    fn pad_image(&self, img: &RgbImage) -> RgbImage {
        let pw = std::cmp::max(img.width(), 32);
        let ph = std::cmp::max(img.height(), 32);
        let mut new_img = RgbImage::from_pixel(pw, ph, Rgb([0_u8, 0_u8, 0_u8]));
        for r in 0..img.width() {
            for c in 0..img.height() {
                let pixel = img.get_pixel(r, c);
                new_img.put_pixel(r, c, pixel.to_owned());
            }
        }
        new_img
    }

    fn normalize(&self, img: &Rgb32FImage) -> Rgb32FImage {
        let w = img.width();
        let h = img.height();
        let mut nimg = Rgb32FImage::new(w, h);
        for x in 0..w {
            for y in 0..h {
                let pixel = img.get_pixel(x, y);
                let [r, g, b] = pixel.0;
                let r = r as f32 * self.alpha[0] + self.beta[0];
                let g = g as f32 * self.alpha[1] + self.beta[1];
                let b = b as f32 * self.alpha[2] + self.beta[2];
                nimg.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        return nimg;
    }

    fn resize(&self, img: &RgbImage) -> Result<PreOutput> {
        let h = img.height();
        let w = img.width();
        let mut ratio = 1.0;
        if std::cmp::max(h, w) as usize > self.limit_side_len {
            if h > w {
                ratio = self.limit_side_len as f32 / h as f32;
            } else {
                ratio = self.limit_side_len as f32 / w as f32;
            }
        }

        let mut rh = (h as f32 * ratio).round() as u32;
        let mut rw = (w as f32 * ratio).round() as u32;

        if std::cmp::max(rw, rh) > self.max_side_limit as u32 {
            ratio = self.max_side_limit as f32 / std::cmp::max(rw, rh) as f32;
            rw = (rw as f32 * ratio).round() as u32;
            rh = (rh as f32 * ratio).round() as u32;
        }

        rh = std::cmp::max(((rh as f32 / 32.0).round() * 32.0) as u32, 32);
        rw = std::cmp::max(((rw as f32 / 32.0).round() * 32.0) as u32, 32);
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
            ratio_w: w as f32 / rw as f32,
            ratio_h: h as f32 / rh as f32,
            input: convert_rgb_to_rgb32f(&rimg),
        };
        return Ok(result);
    }
}
