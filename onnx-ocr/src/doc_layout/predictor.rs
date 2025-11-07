use std::{cell::RefCell, path::Path, rc::Rc};

use anyhow::Result;
use image::RgbImage;
use ndarray::Ix2;
use ort::{inputs, session::Session, value::Tensor, value::TensorRef};

use crate::{
    common::{imgproc::load_image, onnx::load_session},
    doc_layout::{postprocess::PostProcessor, preprocess::PreProcessor},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutLabel {
    ParaGraphTitle,
    Image,
    Text,
    Number,
    Abstract,
    Content,
    FigureTitle,
    Formula,
    Table,
    Reference,
    DocTitle,
    Footnote,
    Header,
    Algorithm,
    Footer,
    Seal,
    Chart,
    FormulaNumber,
    AsideText,
    ReferenceContent,
}

impl LayoutLabel {
    pub fn new_from_str(label: &str) -> Self {
        match label {
            "paragraph_title" => LayoutLabel::ParaGraphTitle,
            "image" => LayoutLabel::Image,
            "text" => LayoutLabel::Text,
            "number" => LayoutLabel::Number,
            "abstract" => LayoutLabel::Abstract,
            "content" => LayoutLabel::Content,
            "figure_title" => LayoutLabel::FigureTitle,
            "formula" => LayoutLabel::Formula,
            "table" => LayoutLabel::Table,
            "reference" => LayoutLabel::Reference,
            "doc_title" => LayoutLabel::DocTitle,
            "footnote" => LayoutLabel::Footnote,
            "header" => LayoutLabel::Header,
            "algorithm" => LayoutLabel::Algorithm,
            "footer" => LayoutLabel::Footer,
            "seal" => LayoutLabel::Seal,
            "chart" => LayoutLabel::Chart,
            "formula_number" => LayoutLabel::FormulaNumber,
            "aside_text" => LayoutLabel::AsideText,
            "reference_content" => LayoutLabel::ReferenceContent,
            _ => {
                panic!("invalid layout label ");
            }
        }
    }
    pub fn as_str(&self) -> &str {
        match self {
            LayoutLabel::ParaGraphTitle => "paragraph_title",
            LayoutLabel::Image => "image",
            LayoutLabel::Text => "text",
            LayoutLabel::Number => "number",
            LayoutLabel::Abstract => "abstract",
            LayoutLabel::Content => "content",
            LayoutLabel::FigureTitle => "figure_title",
            LayoutLabel::Formula => "formula",
            LayoutLabel::Table => "table",
            LayoutLabel::Reference => "reference",
            LayoutLabel::DocTitle => "doc_title",
            LayoutLabel::Footnote => "footnote",
            LayoutLabel::Header => "header",
            LayoutLabel::Algorithm => "algorithm",
            LayoutLabel::Footer => "footer",
            LayoutLabel::Seal => "seal",
            LayoutLabel::Chart => "chart",
            LayoutLabel::FormulaNumber => "formula_number",
            LayoutLabel::AsideText => "aside_text",
            LayoutLabel::ReferenceContent => "reference_content",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub label: LayoutLabel,
    pub coordinate: [f32; 4],
    pub score: f32,
}

impl LayoutResult {
    pub fn area(&self) -> f32 {
        let [x1, y1, x2, y2] = self.coordinate;
        let width = x2 - x1;
        let height = y2 - y1;
        width * height
    }
}

pub struct LayoutPredictor {
    sess: Rc<RefCell<Session>>,
    pre_processor: PreProcessor,
    post_processor: PostProcessor,
}

impl LayoutPredictor {
    pub fn try_new<P: AsRef<Path>>(model_path: P) -> Result<Self> {
        let sess = Rc::new(RefCell::new(load_session(model_path)?));
        let pre_processor = PreProcessor::default();
        let post_processor = PostProcessor::default();
        Ok(Self {
            sess,
            pre_processor,
            post_processor,
        })
    }

    pub fn predict_path<P: AsRef<Path>>(&mut self, img_path: P) -> Result<Vec<LayoutResult>> {
        let img = load_image(img_path)?;
        self.predict_image(&img)
    }

    pub fn predict_image(&self, img: &RgbImage) -> Result<Vec<LayoutResult>> {
        let img_width = img.width();
        let img_height = img.height();
        let pre_output = self.pre_processor.process(&img)?;
        let input = pre_output.get_input_as_ndarray();
        let mut sess = self.sess.borrow_mut();
        let outputs =sess
            .run(inputs![
                "image" =>TensorRef::from_array_view(&input).unwrap(), 
                "im_shape"=>Tensor::from_array(([1, 2],vec![800.0_f32,800.0])).unwrap(),
                "scale_factor"=>Tensor::from_array(([1, 2],vec![pre_output.ratio_h,pre_output.ratio_w])).unwrap()]
            ).unwrap();

        let output = outputs["fetch_name_0"].try_extract_array::<f32>().unwrap();

        let preds = output.squeeze();
        let bitmap = preds.into_dimensionality::<Ix2>()?.to_owned();
        let boxes_result = self.post_processor.process(&bitmap)?;
        let mut results = Vec::new();
        let area_thresh = if img_width > img_height { 0.82 } else { 0.93 };
        let img_area = (img_width * img_height) as f32;
        for obj in boxes_result.iter() {
            if obj.label == "image" {
                let [minx, maxx, miny, maxy] = obj.coordinate;
                let area = (maxx - minx) * (maxy - miny);
                if area >= img_area * area_thresh {
                    continue;
                }
            }
            let det_res = LayoutResult {
                label: LayoutLabel::new_from_str(&obj.label),
                coordinate: obj.coordinate,
                score: obj.score,
            };
            results.push(det_res);
        }
        Ok(results)
    }
}
