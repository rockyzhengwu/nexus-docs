use image::{
    Rgb, Rgb32FImage, RgbImage,
    imageops::{FilterType, crop_imm, resize},
};

pub struct PreProcessor {
    alpha: [f32; 3],
    beta: [f32; 3],
}

impl Default for PreProcessor {
    fn default() -> Self {
        let alpha = [1.0 / 0.229, 1.0 / 0.224, 1.0 / 0.225];
        let beta = [-0.485 / 0.229, -0.456 / 0.224, -0.406 / 0.225];
        PreProcessor { alpha, beta }
    }
}

impl PreProcessor {
    pub fn process(&self, img: &RgbImage) -> Rgb32FImage {
        let img = self.resize(img);
        let img = self.crop(&img);
        let img = self.normalize(&img);
        img
    }

    fn resize(&self, img: &RgbImage) -> RgbImage {
        let target_short_edge: u32 = 256;
        let w = img.width();
        let h = img.height();
        let min_edge = h.min(w);
        let scale = target_short_edge as f32 / min_edge as f32;
        let rh = (h as f32 * scale).round() as u32;
        let rw = (w as f32 * scale).round() as u32;
        let rimg = resize(img, rw, rh, FilterType::Triangle);
        rimg
    }

    fn crop(&self, img: &RgbImage) -> RgbImage {
        let cw = 224;
        let ch = 224;
        let w = img.width();
        let h = img.height();
        let x1 = if w < cw { 0 } else { (w - cw) / 2 };
        let y1 = if h < ch { 0 } else { (h - ch) / 2 };
        let cimg = crop_imm(img, x1, y1, cw, ch).to_image();
        cimg
    }

    fn normalize(&self, img: &RgbImage) -> Rgb32FImage {
        let w = img.width();
        let h = img.height();
        let mut nimg = Rgb32FImage::new(w, h);
        for x in 0..w {
            for y in 0..h {
                let pixel = img.get_pixel(x, y);
                let [r, g, b] = pixel.0;
                let r = (r as f32) / 255.0 * self.alpha[0] + self.beta[0];
                let g = (g as f32) / 255.0 * self.alpha[1] + self.beta[1];
                let b = (b as f32) / 255.0 * self.alpha[2] + self.beta[2];
                nimg.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        nimg
    }
}
