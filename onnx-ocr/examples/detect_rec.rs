use image::{Rgb, RgbImage};
use imageproc::drawing::draw_line_segment_mut;
use imageproc::geometric_transformations::{Interpolation, warp_into};
use onnx_ocr::{
    common::imgproc::load_image, common::quad::Quad,
    text_detection::predictor::TextDetectionPredictor,
    text_recognition::predictor::TextRecognitionPredictor,
};

use std::time::{Duration, SystemTime};

fn draw_result(img: RgbImage, bboxs: &[Quad]) {
    let mut img = img;
    let red = Rgb([255_u8, 0_u8, 0_u8]);
    for bbox in bboxs.iter() {
        draw_line_segment_mut(
            &mut img,
            (bbox.tl.x as f32, bbox.tl.y as f32),
            (bbox.tr.x as f32, bbox.tr.y as f32),
            red,
        );

        draw_line_segment_mut(
            &mut img,
            (bbox.tr.x as f32, bbox.tr.y as f32),
            (bbox.dr.x as f32, bbox.dr.y as f32),
            red,
        );
        draw_line_segment_mut(
            &mut img,
            (bbox.dr.x as f32, bbox.dr.y as f32),
            (bbox.dl.x as f32, bbox.dl.y as f32),
            red,
        );

        draw_line_segment_mut(
            &mut img,
            (bbox.dl.x as f32, bbox.dl.y as f32),
            (bbox.tl.x as f32, bbox.tl.y as f32),
            red,
        );
    }
    img.save("texxt_detect_result.jpg").unwrap();
}

fn main() {
    //tracing_subscriber::fmt::init();
    let model_path = "/home/zhengwu/workspace/private/projects/paddle-ocr-onnx/onnx/PP-OCRv5_server_det/model.onnx";
    let img_path = "/home/zhengwu/workspace/private/projects/paddle-ocr-onnx/general_ocr_002.jpeg";
    let mut detect_predictor = TextDetectionPredictor::try_new(model_path).unwrap();
    let mut rec_predictor = TextRecognitionPredictor::try_new(
        "/home/zhengwu/workspace/private/projects/paddle-ocr-onnx/onnx/PP-OCRv5_server_rec/model.onnx",
        "/home/zhengwu/workspace/private/projects/paddle-ocr-onnx/character.json",
    ).unwrap();

    let start = SystemTime::now();
    let mut n = 0;
    let mut total_det = Duration::new(0, 0);
    let mut total_rec = Duration::new(0, 0);
    let mut total = 0;
    while n < 10 {
        let det_start = SystemTime::now();
        let result = detect_predictor.predict_path(img_path).unwrap();
        let det_end = SystemTime::now();
        total_det += det_end.duration_since(det_start).expect("time error");

        let img = load_image(img_path).unwrap();
        let mut images = Vec::new();
        for bbox in result.bboxs.iter() {
            if let Some(projection) = bbox.projection() {
                let mut dest = RgbImage::new(bbox.width.round() as u32, bbox.height.round() as u32);
                //let warp_img = warp(&img, &projection, Interpolation::Bilinear, Rgb([255, 0, 0]));
                warp_into(
                    &img,
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
        let rec_start = SystemTime::now();
        let res = rec_predictor.predict(images).unwrap();
        let rec_end = SystemTime::now();
        total_rec += rec_end.duration_since(rec_start).expect("time error");
        n += 1;
    }
    let end = SystemTime::now();
    let total = end.duration_since(start).expect("Time went backwards");
    println!("total cost: {:?}", total);
    println!("det coset:{:?}", total_det);
    println!("rec_cost: {:?}", total_rec);
}
