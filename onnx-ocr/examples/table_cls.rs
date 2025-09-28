use onnx_ocr::table_cls::predictor::TableClsPredictor;

fn main() {
    let model_path = "/home/zhengwu/workspace/private/projects/nexus-docs/onnx-ocr/onnx/PP-LCNet_x1_0_table_cls/model.onnx";
    let img_path = "/home/zhengwu/workspace/private/projects/nexus-docs/onnx-ocr/test_images/table_recognition.jpg";
    let mut detect_predictor = TableClsPredictor::try_new(model_path).unwrap();
    let result = detect_predictor.predict_path(img_path).unwrap();
    println!("{:?}", result);
}
