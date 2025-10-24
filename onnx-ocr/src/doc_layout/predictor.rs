use std::{cell::RefCell, path::Path, rc::Rc};

use anyhow::Result;
use image::RgbImage;
use ndarray::Ix2;
use ort::{inputs, session::Session, value::Tensor, value::TensorRef};

use crate::{
    common::{imgproc::load_image, onnx::load_session},
    doc_layout::{postprocess::PostProcessor, preprocess::PreProcessor},
};

#[derive(Debug)]
pub struct LayoutResult {
    pub label: String,
    pub coordinate: [f32; 4],
    pub score: f32,
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
                label: obj.label.clone(),
                coordinate: obj.coordinate,
                score: obj.score,
            };
            results.push(det_res);
        }
        Ok(results)
    }
}
