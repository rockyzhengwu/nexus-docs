use std::fmt::format;

pub struct Settings {
    pub text_det_model_path: String,
    pub text_rec_model_path: String,
    pub text_charactor_list_path: String,
    pub wired_table_cell_det_model_path: String,
    pub wireless_table_cell_det_model_path: String,
    pub doc_layout_model_path: String,
}

impl Settings {
    pub fn new() -> Self {
        let model_base_path = "/home/zhengwu/workspace/private/projects/nexus-docs/onnx-ocr/onnx";
        let text_det_model_path = format!("{}/PP-OCRv5_server_det/model.onnx", model_base_path);
        let text_rec_model_path = format!("{}/PP-OCRv5_server_rec/model.onnx", model_base_path);

        let text_charactor_list_path =
            format!("{}/PP-OCRv5_server_rec/character.json", model_base_path);

        let doc_layout_model_path =
            format!("{}/pp-DocLayout_plus-L_infer/model.onnx", model_base_path);

        let wired_table_cell_det_model_path = format!(
            "{}/RT-DETR-L_wired_table_cell_det/model.onnx",
            model_base_path
        );
        let wireless_table_cell_det_model_path = format!(
            "{}/RT-DETR-L_wireless_table_cell_det/model.onnx",
            model_base_path
        );
        return Settings {
            text_det_model_path,
            text_rec_model_path,
            text_charactor_list_path,
            doc_layout_model_path,
            wired_table_cell_det_model_path,
            wireless_table_cell_det_model_path,
        };
    }
}
