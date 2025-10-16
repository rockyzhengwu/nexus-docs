use crate::{
    doc_layout::predictor::LayoutPredictor,
    settings::{self, Settings},
    table_cell_detection::predictor::TableCellDetector,
    table_cls::predictor::TableClsPredictor,
    table_structure::predictor::TableStructurePredictor,
    text_detection::predictor::TextDetectionPredictor,
    text_recognition::predictor::TextRecognitionPredictor,
};
use anyhow::Result;

pub struct ModelContext {
    settings: Settings,
    pub text_det_predictor: TextDetectionPredictor,
    pub text_rec_predictor: TextRecognitionPredictor,
    pub layout_predictor: LayoutPredictor,
    pub table_cls_predictor: TableClsPredictor,
    pub wired_table_cell_predictor: TableCellDetector,
    pub wireless_table_cell_predictor: TableCellDetector,
    pub wired_table_structure_predictor: TableStructurePredictor,
    pub wireless_table_structure_predictor: TableStructurePredictor,
}

impl ModelContext {
    pub fn new() -> Result<Self> {
        let settings = Settings::new();
        let text_det_predictor =
            TextDetectionPredictor::try_new(settings.text_det_model_path.as_str())?;
        let text_rec_predictor = TextRecognitionPredictor::try_new(
            settings.text_rec_model_path.as_str(),
            settings.text_charactor_list_path.as_str(),
        )?;
        let layout_predictor = LayoutPredictor::try_new(settings.doc_layout_model_path.as_str())?;
        let table_cls_predictor =
            TableClsPredictor::try_new(settings.table_cls_model_path.as_str())?;
        let wired_table_cell_predictor =
            TableCellDetector::try_new(settings.wired_table_cell_det_model_path.as_str())?;
        let wireless_table_cell_predictor =
            TableCellDetector::try_new(settings.wireless_table_cell_det_model_path.as_str())?;
        let wired_table_structure_predictor = TableStructurePredictor::try_new(
            settings.wired_table_structure_model_path.as_str(),
            settings.wired_table_structure_character_path.as_str(),
        )?;
        let wireless_table_structure_predictor = TableStructurePredictor::try_new(
            settings.wireless_table_structure_model_path.as_str(),
            settings.wireless_table_structure_character_path.as_str(),
        )?;

        Ok(Self {
            settings,
            text_det_predictor,
            text_rec_predictor,
            layout_predictor,
            table_cls_predictor,
            wired_table_cell_predictor,
            wireless_table_cell_predictor,
            wired_table_structure_predictor,
            wireless_table_structure_predictor,
        })
    }
}
