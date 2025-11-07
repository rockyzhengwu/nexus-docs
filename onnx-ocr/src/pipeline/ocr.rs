use crate::{common::quad::Quad, model_context::ModelContext};
use anyhow::Result;
use image::{Rgb, RgbImage, imageops::rotate90};
use imageproc::geometric_transformations::{Interpolation, warp_into};

#[derive(Debug, Clone)]
pub struct OcrResultItem {
    pub polys: Quad,
    pub content: String,
    pub bbox: [f32; 4],
}

impl OcrResultItem {
    pub fn new(polys: Quad, content: String, bbox: [f32; 4]) -> Self {
        OcrResultItem {
            polys,
            content,
            bbox,
        }
    }
}

pub fn ocr(context: &ModelContext, img: &RgbImage) -> Result<Vec<OcrResultItem>> {
    let detect_predictor = &context.text_det_predictor;
    let rec_predictor = &context.text_rec_predictor;
    let result = detect_predictor.predict_image(img)?;
    let mut images = Vec::new();
    let mut n = 0;
    for poly in result.polys.iter() {
        if let Some(projection) = poly.projection() {
            let mut dest = RgbImage::new(poly.width.ceil() as u32, poly.height.ceil() as u32);
            warp_into(
                img,
                &projection,
                Interpolation::Bilinear,
                Rgb([0, 0, 0]),
                &mut dest,
            );
            let height = dest.height();
            let width = dest.width();
            // TODO fix this roate 90 click wise can rotate -90
            if (height as f32 / width as f32) > 3.0 {
                let rotated_dest = rotate90(&dest);
                images.push(rotated_dest);
            } else {
                images.push(dest);
            }
            n += 1;
        } else {
            println!("projection is none");
        }
    }
    let texts = rec_predictor.predict(images)?;
    let mut ocr_items = Vec::new();
    for i in 0..texts.len() {
        let poly = &result.polys[i];
        let content = &texts[i].0;

        let item = OcrResultItem {
            polys: poly.to_owned(),
            content: content.to_owned(),
            bbox: poly.bbox(),
        };
        ocr_items.push(item);
    }
    Ok(ocr_items)
}
