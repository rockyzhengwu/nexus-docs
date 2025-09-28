use image::{Rgb, RgbImage};
use imageproc::drawing::draw_line_segment_mut;
use onnx_ocr::common::imgproc::load_image;
use onnx_ocr::table_cell_detection::predictor::{DetectResult, TableCellDetector};

fn main() {
    let model_path = "/home/zhengwu/workspace/private/projects/paddle-ocr-onnx/onnx/RT-DETR-L_wired_table_cell_det/model.onnx";
    // let model_path = "/home/zhengwu/workspace/private/projects/paddle-ocr-onnx/onnx/RT-DETR-L_wireless_table_cell_det/model.onnx";
    let img_path = "/home/zhengwu/workspace/Github/PaddleX/table_recognition.jpg";
    let mut detect_predictor = TableCellDetector::try_new(model_path).unwrap();
    let result = detect_predictor.predict_path(img_path).unwrap();
    let mut img = load_image(img_path).unwrap();
    draw_result(&mut img, result.as_slice());
}

fn draw_result(img: &mut RgbImage, objs: &[DetectResult]) {
    let red = Rgb([255_u8, 0_u8, 0_u8]);
    for item in objs.iter() {
        let [minx, miny, maxx, maxy] = item.coordinate;
        draw_line_segment_mut(img, (minx, miny), (maxx, miny), red);
        draw_line_segment_mut(img, (maxx, miny), (maxx, maxy), red);
        draw_line_segment_mut(img, (maxx, maxy), (minx, maxy), red);
        draw_line_segment_mut(img, (minx, maxy), (minx, miny), red);
    }
    img.save("layout_result.jpg").unwrap();
}
