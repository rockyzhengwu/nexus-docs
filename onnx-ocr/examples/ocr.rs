use image::ImageReader;
use onnx_ocr::{model_context::ModelContext, pipeline::ocr};

pub fn main() {
    let img_path = "/home/zhengwu/workspace/private/projects/paddle-ocr-onnx/general_ocr_002.png";
    let file = std::fs::File::open(img_path).unwrap();
    let reader = std::io::BufReader::new(file);
    let context = ModelContext::new().unwrap();
    let img_reader = ImageReader::new(reader).with_guessed_format().unwrap();
    let original_img = img_reader.decode().unwrap();
    let rgb = original_img.to_rgb8();
    let res = ocr::ocr(&context, &rgb).unwrap();

    println!("{:?}", res);
}
