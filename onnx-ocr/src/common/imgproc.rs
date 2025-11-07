use std::io::BufReader;
use std::path::Path;

use anyhow::Result;
use image::{GenericImage, RgbImage};
use image::{ImageBuffer, ImageReader, Pixel, Rgb, Rgb32FImage};
use imageproc::definitions::Image;
use imageproc::point::Point;

pub fn load_image<P: AsRef<Path>>(img_path: P) -> Result<RgbImage> {
    let img_reader = BufReader::new(std::fs::File::open(img_path).unwrap());
    let reader = ImageReader::new(img_reader).with_guessed_format()?;
    let original_img = reader.decode()?;
    let rgb = original_img.to_rgb8();
    Ok(rgb)
}

pub fn convert_rgb_to_rgb32f(rgb_image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Rgb32FImage {
    let (width, height) = rgb_image.dimensions();
    let mut rgb32f_image = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel_u8 = rgb_image.get_pixel(x, y);
            let r_f32 = pixel_u8[0] as f32 / 255.0;
            let g_f32 = pixel_u8[1] as f32 / 255.0;
            let b_f32 = pixel_u8[2] as f32 / 255.0;
            rgb32f_image.put_pixel(x, y, Rgb([b_f32, g_f32, r_f32]));
        }
    }
    rgb32f_image
}

pub fn pointu32_to_pintf32(point: &Point<u32>) -> Point<f32> {
    Point::new(point.x as f32, point.y as f32)
}

pub fn rotate_clock_wise_90<P: Pixel>(image: &Image<P>) -> Image<P> {
    let width = image.width();
    let height = image.height();
    let mut dest = ImageBuffer::new(height, width);
    for r in 0..width {
        for c in 0..height {
            let pixel = image.get_pixel(r, c);
            dest.put_pixel(height - c - 1, r, pixel.to_owned());
        }
    }
    dest
}
