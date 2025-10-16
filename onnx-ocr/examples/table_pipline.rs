use image::ImageReader;
use onnx_ocr::{model_context::ModelContext, pipeline::table};

pub fn main() {
    let img_path = "/home/zhengwu/workspace/private/projects/nexus-docs/onnx-ocr/test_images/table_recognition_v2.jpg";
    let file = std::fs::File::open(img_path).unwrap();
    let reader = std::io::BufReader::new(file);
    let context = ModelContext::new().unwrap();
    let img_reader = ImageReader::new(reader).with_guessed_format().unwrap();
    let original_img = img_reader.decode().unwrap();
    let rgb = original_img.to_rgb8();
    let res = table::extract_table(&context, &rgb).unwrap();

    println!("html:{:?}", res.to_html());
}
