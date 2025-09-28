use image::{Rgb, RgbImage};
use imageproc::drawing::draw_line_segment_mut;
use onnx_ocr::doc_layout::predictor::{DetectResult, LayoutPredictor};
use onnx_ocr::pdf::parser::pdf_page_to_image;
use std::fs::File;
use std::io::Read;

fn main() {
    //let path = "/home/zhengwu/Documents/pdf.pdf";
    let path = "/home/zhengwu/workspace/Github/MinerU/demo/pdfs/demo1.pdf";
    let mut file = File::open(path).unwrap();
    let mut pdf_content = Vec::new();
    file.read_to_end(&mut pdf_content).unwrap();
    let mut image = pdf_page_to_image(pdf_content.as_slice(), 4, None).unwrap();
    image.save("table_example2.png").unwrap();

    let model_path = "/home/zhengwu/workspace/private/projects/paddle-ocr-onnx/onnx/pp-DocLayout_plus-L_infer/model.onnx";
    let mut detect_predictor = LayoutPredictor::try_new(model_path).unwrap();
    let result = detect_predictor.predict_image(&image).unwrap();
    draw_result(&mut image, result.as_slice());
    //image.save("page_page.png").unwrap();
}

fn draw_result(img: &mut RgbImage, objs: &[DetectResult]) {
    let red = Rgb([255_u8, 0_u8, 0_u8]);
    for item in objs.iter() {
        println!("{:?}", item);
        let [minx, miny, maxx, maxy] = item.coordinate;
        draw_line_segment_mut(img, (minx, miny), (maxx, miny), red);
        draw_line_segment_mut(img, (maxx, miny), (maxx, maxy), red);
        draw_line_segment_mut(img, (maxx, maxy), (minx, maxy), red);
        draw_line_segment_mut(img, (minx, maxy), (minx, miny), red);
    }
    img.save("layout_result.jpg").unwrap();
}
