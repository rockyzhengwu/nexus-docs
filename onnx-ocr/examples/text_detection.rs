use image::{Rgb, RgbImage};
use imageproc::drawing::draw_line_segment_mut;
use imageproc::geometric_transformations::{Interpolation, warp, warp_into};
use onnx_ocr::{
    common::imgproc::load_image, common::quad::Quad,
    text_detection::predictor::TextDetectionPredictor,
};

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
    tracing_subscriber::fmt::init();

    let model_path = "/home/zhengwu/workspace/private/projects/paddle-ocr-onnx/onnx/PP-OCRv5_server_det/model.onnx";
    let img_path = "/home/zhengwu/workspace/private/projects/paddle-ocr-onnx/general_ocr_002.jpeg";
    let mut detect_predictor = TextDetectionPredictor::try_new(model_path).unwrap();
    let result = detect_predictor.predict_path(img_path).unwrap();
    let mut img = load_image(img_path).unwrap();
    let mut i = 0;

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
            dest.save(format!("warp_result_{}.png", i)).unwrap();
            i += 1;
        } else {
            println!("projection is none");
        }
    }
    img.save("texxt_detect_result.jpg").unwrap();
}
