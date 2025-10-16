use crate::common::imgproc::convert_rgb_to_rgb32f;
use anyhow::Result;
use image::{Rgb, Rgb32FImage, RgbImage, imageops::FilterType, imageops::resize};
use ndarray::Array4;

pub struct PreProcessor {
    target_long_edge: u32,
    alpha: [f32; 3],
    beta: [f32; 3],
}

pub struct PreOutput {
    ori_shape: [u32; 2],
    img: Rgb32FImage,
}

impl PreOutput {
    pub fn ori_shape(&self) -> &[u32; 2] {
        &self.ori_shape
    }

    pub fn input(&self) -> Array4<f32> {
        let mut input = Array4::<f32>::zeros((1, 3, 512, 512));
        for y in 0..512 {
            for x in 0..512 {
                let pixel = self.img.get_pixel(x as u32, y as u32);
                let [r, g, b] = pixel.0;
                input[[0, 0, y, x]] = r;
                input[[0, 1, y, x]] = g;
                input[[0, 2, y, x]] = b;
            }
        }
        input
    }
}

impl Default for PreProcessor {
    fn default() -> Self {
        let alpha = [1.0 / 0.229, 1.0 / 0.224, 1.0 / 0.225];
        let beta = [-0.485 / 0.229, -0.456 / 0.224, -0.406 / 0.225];
        let target_long_edge = 512;
        PreProcessor {
            target_long_edge,
            alpha,
            beta,
        }
    }
}

impl PreProcessor {
    pub fn process(&self, img: &RgbImage) -> Result<PreOutput> {
        let w = img.width();
        let h = img.height();
        let rimg = self.resize(img)?;
        let nimg = self.normalize(&rimg);
        let pimg = self.pad(&nimg);
        let out = PreOutput {
            ori_shape: [w, h],
            img: pimg,
        };
        Ok(out)
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

    fn resize(&self, img: &RgbImage) -> Result<Rgb32FImage> {
        let h = img.height();
        let w = img.width();
        let scale = self.target_long_edge as f32 / (h.max(w) as f32);
        let rh = (h as f32 * scale).floor() as u32;
        let rw = (w as f32 * scale).floor() as u32;
        let rimg = resize(img, rw, rh, FilterType::Triangle);
        let rimg = convert_rgb_to_rgb32f(&rimg);
        return Ok(rimg);
    }

    fn pad(&self, img: &Rgb32FImage) -> Rgb32FImage {
        let h = img.height();
        let w = img.width();
        let mut new_img = Rgb32FImage::from_pixel(512, 512, Rgb([0.0, 0.0, 0.0]));
        for r in 0..w {
            for c in 0..h {
                let pixel = img.get_pixel(r, c);
                new_img.put_pixel(r, c, pixel.to_owned());
            }
        }
        return new_img;
    }
}
