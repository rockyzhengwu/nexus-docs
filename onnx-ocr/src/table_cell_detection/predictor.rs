use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use anyhow::Result;
use image::RgbImage;

use ndarray::Ix2;
use ort::{
    inputs,
    session::Session,
    value::{Tensor, TensorRef},
};

use crate::{
    common::{imgproc::load_image, onnx::load_session},
    table_cell_detection::{postprocess::PostProcessor, preprocess::PreProcessor},
};

#[derive(Debug, Clone)]
pub struct TableCelltResult {
    pub label: String,
    pub coordinate: [f32; 4],
    pub score: f32,
}

pub struct TableCellDetector {
    sess: Rc<RefCell<Session>>,
    pre_processor: PreProcessor,
    post_processor: PostProcessor,
}

impl TableCellDetector {
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

    pub fn predict_path<P: AsRef<Path>>(&mut self, img_path: P) -> Result<Vec<TableCelltResult>> {
        let img = load_image(img_path)?;
        self.predict_image(&img)
    }

    pub fn predict_image(&self, img: &RgbImage) -> Result<Vec<TableCelltResult>> {
        let img_width = img.width();
        let img_height = img.height();
        let pre_output = self.pre_processor.process(img)?;
        let input = pre_output.get_input_as_ndarray();
        let mut sess = self.sess.borrow_mut();

        let outputs =
            sess
            .run(inputs![
                "image" =>TensorRef::from_array_view(&input).unwrap(), 
                "im_shape"=>Tensor::from_array(([1, 2],vec![640.0_f32,640.0])).unwrap(),
                "scale_factor"=>Tensor::from_array(([1, 2],vec![pre_output.ratio_h,pre_output.ratio_w])).unwrap()]
            ).unwrap();

        let output = outputs["fetch_name_0"].try_extract_array::<f32>().unwrap();
        let preds = output.squeeze();
        let bitmap = preds.into_dimensionality::<Ix2>()?.to_owned();
        let boxes_result =
            self.post_processor
                .process(&bitmap, img_width as f32, img_height as f32)?;
        let mut results = Vec::new();
        for obj in boxes_result.iter() {
            let det_res = TableCelltResult {
                label: "cell".to_string(),
                coordinate: obj.coordinate,
                score: obj.score,
            };
            results.push(det_res);
        }
        Ok(results)
    }
}
