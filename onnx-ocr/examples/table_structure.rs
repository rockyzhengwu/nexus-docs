use onnx_ocr::table_structure::predictor::TableStructurePredictor;

fn main() {
    let model_path = "/home/zhengwu/workspace/private/projects/nexus-docs/onnx-ocr/onnx/SLANeXt_wired/model.onnx";
    let img_path = "/home/zhengwu/workspace/private/projects/nexus-docs/onnx-ocr/test_images/table_recognition_v2.jpg";
    let character_path = "/home/zhengwu/workspace/private/projects/nexus-docs/onnx-ocr/onnx/SLANeXt_wired/character.json";
    let mut detect_predictor =
        TableStructurePredictor::try_new(model_path, character_path).unwrap();
    let result = detect_predictor.predict_path(img_path).unwrap();
    println!("{:?}", result);
}
