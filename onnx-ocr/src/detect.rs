use anyhow::Result;
use image::{
    Rgb, Rgb32FImage,
    imageops::{FilterType, resize},
};

pub struct DetPreProcess {
    max_side_limit: usize,
    min_side_limit: usize,
    mean: [f32; 3],
    std: [f32; 3],
    alpha: [f32; 3],
    beta: [f32; 3],
}

impl Default for DetPreProcess {
    fn default() -> Self {
        let mean = [0.485, 0.456, 0.406];
        let std = [0.229, 0.224, 0.225];
        let alpha = [1.0 / 0.229, 1.0 / 0.224, 1.0 / 0.225];
        let beta = [-0.485 / 0.229, -0.465 / 0.224, -0.406 / 0.225];

        DetPreProcess {
            max_side_limit: 4000,
            min_side_limit: 64,
            mean,
            std,
            alpha,
            beta,
        }
    }
}
pub struct PreProcessResult {
    pub img: Rgb32FImage,
    pub ratio_h: f32,
    pub ratio_w: f32,
    pub width: u32,
    pub height: u32,
}

impl DetPreProcess {
    pub fn process(&self, img: Rgb32FImage) -> Result<PreProcessResult> {
        let w = img.width();
        let h = img.height();
        let paded_image = if (w + h) < self.min_side_limit as u32 {
            self.pad_image(img)
        } else {
            img
        };
        let mut result = self.resize(&paded_image);
        let nimg = self.normalize(&result.img);
        result.img = nimg;
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
                let r = r * self.alpha[0] + self.beta[0];
                let g = g * self.alpha[1] + self.beta[1];
                let b = b * self.alpha[2] + self.beta[2];
                nimg.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        return nimg;
    }

    fn resize(&self, img: &Rgb32FImage) -> PreProcessResult {
        let h = img.height();
        let w = img.width();
        let mut ratio = 1.0;
        if std::cmp::min(h, w) < self.min_side_limit as u32 {
            if h < w {
                ratio = self.min_side_limit as f32 / h as f32;
            } else {
                ratio = self.min_side_limit as f32 / w as f32;
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
            let result = PreProcessResult {
                img: img.to_owned(),
                width: rw,
                height: rh,
                ratio_h: 1.0,
                ratio_w: 1.0,
            };
            return result;
        }
        let rimg = resize(img, rw, rh, FilterType::Triangle);
        let result = PreProcessResult {
            img: rimg,
            width: rw,
            height: rh,
            ratio_h: w as f32 / rw as f32,
            ratio_w: h as f32 / rh as f32,
        };
        result
    }

    fn pad_image(&self, img: Rgb32FImage) -> Rgb32FImage {
        let pw = std::cmp::max(img.width(), 32);
        let ph = std::cmp::max(img.height(), 32);
        let mut new_img = Rgb32FImage::from_pixel(pw, ph, Rgb([0.0, 0.0, 0.0]));
        for r in 0..img.width() {
            for c in 0..img.height() {
                let pixel = img.get_pixel(r, c);
                new_img.put_pixel(r, c, pixel.to_owned());
            }
        }
        new_img
    }
}
