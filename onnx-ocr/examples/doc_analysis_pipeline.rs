use image::ImageReader;
use onnx_ocr::model_context::ModelContext;
use onnx_ocr::pipeline::doc_analysise::doc_analysise;

fn main() {
    let img_path =
        "/home/zhengwu/workspace/private/projects/nexus-docs/onnx-ocr/test_images/table_layout.png";
    let file = std::fs::File::open(img_path).unwrap();
    let reader = std::io::BufReader::new(file);
    let context = ModelContext::new().unwrap();
    let img_reader = ImageReader::new(reader).with_guessed_format().unwrap();
    let original_img = img_reader.decode().unwrap();
    let rgb_img = original_img.to_rgb8();

    let res = doc_analysise(&context, &rgb_img).unwrap();
    println!("main");
}
