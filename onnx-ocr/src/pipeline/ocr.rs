use crate::{common::quad::Quad, model_context::ModelContext};
use anyhow::Result;
use image::{Rgb, RgbImage};
use imageproc::geometric_transformations::{Interpolation, warp_into};

#[derive(Debug)]
pub struct OcrResultItem {
    pub bbox: Quad,
    pub content: String,
}

pub fn ocr(context: &ModelContext, img: &RgbImage) -> Result<Vec<OcrResultItem>> {
    let detect_predictor = &context.text_det_predictor;
    let rec_predictor = &context.text_rec_predictor;
    let result = detect_predictor.predict_image(img)?;
    let mut images = Vec::new();
    for bbox in result.bboxs.iter() {
        if let Some(projection) = bbox.projection() {
            let mut dest = RgbImage::new(bbox.width.round() as u32, bbox.height.round() as u32);
            warp_into(
                img,
                &projection,
                Interpolation::Bilinear,
                Rgb([0, 0, 0]),
                &mut dest,
            );
            images.push(dest);
        } else {
            println!("projection is none");
        }
    }
    let texts = rec_predictor.predict(images)?;
    let mut ocr_items = Vec::new();
    for i in 0..texts.len() {
        let bbox = &result.bboxs[i];
        let content = &texts[i].0;
        let item = OcrResultItem {
            bbox: bbox.to_owned(),
            content: content.to_owned(),
        };
        ocr_items.push(item);
    }
    Ok(ocr_items)
}
