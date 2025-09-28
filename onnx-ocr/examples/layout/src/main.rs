use ab_glyph::{FontRef, PxScale};
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_line_segment_mut, draw_text_mut};
use onnx_ocr::common::imgproc::load_image;
use onnx_ocr::doc_layout::predictor::{DetectResult, LayoutPredictor};

fn main() {
    let model_path = "/home/zhengwu/workspace/private/projects/paddle-ocr-onnx/onnx/pp-DocLayout_plus-L_infer/model.onnx";

    let img_path =
        "/home/zhengwu/workspace/private/projects/nexus-docs/onnx-ocr/test_images/table_layout.png";
    let mut detect_predictor = LayoutPredictor::try_new(model_path).unwrap();
    let result = detect_predictor.predict_path(img_path).unwrap();
    let mut img = load_image(img_path).unwrap();
    draw_result(&mut img, result.as_slice());
}

fn draw_result(img: &mut RgbImage, objs: &[DetectResult]) {
    let font_data = include_bytes!(
        "/home/zhengwu/workspace/private/projects/nexus-docs/onnx-ocr/fonts/PingFang Regular.ttf"
    );
    let red = Rgb([255_u8, 0_u8, 0_u8]);
    let font = FontRef::try_from_slice(font_data).unwrap();
    let scale = PxScale::from(50.0);
    for item in objs.iter() {
        let [minx, miny, maxx, maxy] = item.coordinate;
        let label = item.label.as_str();
        draw_line_segment_mut(img, (minx, miny), (maxx, miny), red);
        draw_line_segment_mut(img, (maxx, miny), (maxx, maxy), red);
        draw_line_segment_mut(img, (maxx, maxy), (minx, maxy), red);
        draw_line_segment_mut(img, (minx, maxy), (minx, miny), red);
        draw_text_mut(
            img,
            Rgb([0_u8, 255_u8, 0_u8]),
            minx.round() as i32,
            miny.round() as i32,
            scale,
            &font,
            label,
        );
    }
    img.save("layout_result.jpg").unwrap();
}
