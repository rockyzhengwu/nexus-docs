use crate::{
    doc_layout::predictor::LayoutPredictor,
    settings::{self, Settings},
    text_detection::predictor::TextDetectionPredictor,
    text_recognition::predictor::TextRecognitionPredictor,
};
use anyhow::Result;

pub struct ModelContext {
    settings: Settings,
    pub text_det_predictor: TextDetectionPredictor,
    pub text_rec_predictor: TextRecognitionPredictor,
    pub layout_predictor: LayoutPredictor,
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

        Ok(Self {
            settings,
            text_det_predictor,
            text_rec_predictor,
            layout_predictor,
        })
    }
}
