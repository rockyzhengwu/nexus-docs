use crate::text_recognition::predictor::TextRecognitionPredictor;
use crate::{common::quad::Quad, text_detection::predictor::TextDetectionPredictor};
use anyhow::Result;
use image::{ImageReader, Rgb, RgbImage};
use imageproc::geometric_transformations::{Interpolation, warp_into};
use std::io::{BufReader, Cursor};

#[derive(Debug)]
pub struct OcrResultItem {
    pub bbox: Quad,
    pub content: String,
}

pub fn ocr(bytes: &[u8]) -> Result<Vec<OcrResultItem>> {
    let detect_model_path = "/home/zhengwu/workspace/learn/front/nexus-docs/onnx-ocr/onnx/PP-OCRv5_server_det/model.onnx";

    let mut detect_predictor = TextDetectionPredictor::try_new(detect_model_path)?;
    let mut rec_predictor = TextRecognitionPredictor::try_new(
        "/home/zhengwu/workspace/learn/front/nexus-docs/onnx-ocr/onnx/PP-OCRv5_server_rec/model.onnx",
        "/home/zhengwu/workspace/learn/front/nexus-docs/onnx-ocr/onnx/PP-OCRv5_server_rec/character.json",
    )?;
    let cursor = Cursor::new(bytes);

    let raw_reader = BufReader::new(cursor);
    let reader = ImageReader::new(raw_reader).with_guessed_format()?;
    let original_img = reader.decode()?;
    let rgb_img = original_img.to_rgb8();
    let result = detect_predictor.predict_image(rgb_img.clone())?;
    let mut images = Vec::new();
    for bbox in result.bboxs.iter() {
        if let Some(projection) = bbox.projection() {
            let mut dest = RgbImage::new(bbox.width.round() as u32, bbox.height.round() as u32);
            warp_into(
                &rgb_img,
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
