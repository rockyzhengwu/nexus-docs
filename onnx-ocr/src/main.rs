use image::Rgb;
use imageproc::drawing::draw_line_segment_mut;
use onnx_ocr::{common::imgproc::load_image, text_detection::predictor::TextDetectionPredictor};

fn main() {
    tracing_subscriber::fmt::init();

    let model_path = "/home/zhengwu/workspace/private/projects/paddle-ocr-onnx/onnx/PP-OCRv5_server_det/model.onnx";
    let img_path = "/home/zhengwu/workspace/private/projects/paddle-ocr-onnx/general_ocr_002.jpeg";
    let mut detect_predictor = TextDetectionPredictor::try_new(model_path).unwrap();
    let result = detect_predictor.predict_path(img_path).unwrap();
    let mut img = load_image(img_path).unwrap();
    let red = Rgb([255_u8, 0_u8, 0_u8]);
    for bbox in result.bboxs.iter() {
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
    img.save("detect_result.jpg").unwrap();
}
