use image::{
    Rgb, Rgb32FImage, RgbImage,
    imageops::{FilterType, resize},
};

pub struct PreProcessor {
    rec_image_shape: [u32; 2],
    max_width: u32,
    ratio: f32,
}

impl Default for PreProcessor {
    fn default() -> Self {
        PreProcessor {
            rec_image_shape: [48, 320],
            max_width: 3200,
            ratio: 320.0 / 48.0,
        }
    }
}

impl PreProcessor {
    pub fn process(&self, img: &RgbImage) -> Rgb32FImage {
        let width = img.width();
        let height = img.height();
        let target_h = self.rec_image_shape[0];
        let ratio = (width as f32) / (height as f32);
        let max_wh_ratio = ratio.max(self.ratio);
        let mut target_w = ((target_h as f32) * max_wh_ratio).ceil() as u32;

        if target_w > self.max_width {
            target_w = self.max_width;
            let out_image = resize(img, target_w, target_h, FilterType::Triangle);
            self.normalize(&out_image)
        } else {
            let resized_w = (target_h as f32 * ratio).floor() as u32;
            if resized_w < target_w {
                target_w = resized_w;
            }
            let resized_image = resize(img, target_w, target_h, FilterType::Triangle);
            let mut out_image = RgbImage::from_pixel(target_w, target_h, Rgb([0, 0, 0]));
            for x in 0..target_w {
                for y in 0..target_h {
                    let pixel = resized_image.get_pixel(x, y);
                    out_image.put_pixel(x, y, pixel.to_owned());
                }
            }
            self.normalize(&out_image)
        }
    }

    fn normalize(&self, img: &RgbImage) -> Rgb32FImage {
        let w = img.width();
        let h = img.height();
        let mut nimg = Rgb32FImage::new(w, h);
        for x in 0..w {
            for y in 0..h {
                let pixel = img.get_pixel(x, y);
                let [r, g, b] = pixel.0;
                let r = ((r as f32 / 255.0) - 0.5) / 0.5;
                let g = ((g as f32 / 255.0) - 0.5) / 0.5;
                let b = ((b as f32 / 255.0) - 0.5) / 0.5;
                nimg.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        return nimg;
    }
}
