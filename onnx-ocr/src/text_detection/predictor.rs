use std::{cell::RefCell, path::Path, rc::Rc};

use anyhow::Result;
use image::{Rgb, RgbImage};
use imageproc::{
    geometric_transformations::{Interpolation, warp_into},
    point::Point,
};
use ndarray::Ix2;
use ort::{inputs, session::Session, value::TensorRef};

use crate::{
    common::{imgproc::load_image, onnx::load_session, quad::Quad},
    text_detection::{postprocess::PostProcessor, preprocess::PreProcessor},
};

#[derive(Debug)]
pub struct DetectResult {
    pub bboxs: Vec<Quad>,
    pub scores: Vec<f32>,
}

impl DetectResult {
    pub fn crop_text_object(&self, img: RgbImage) -> Vec<RgbImage> {
        let mut res = Vec::new();
        for bbox in self.bboxs.iter() {
            if let Some(proj) = bbox.projection() {
                let mut dest = RgbImage::new(bbox.width.round() as u32, bbox.height.round() as u32);
                warp_into(
                    &img,
                    &proj,
                    Interpolation::Bilinear,
                    Rgb([0, 0, 0]),
                    &mut dest,
                );
                res.push(dest);
            } else {
                // TODO proj is None
                println!("Waring: Projection is None");
            }
        }
        res
    }
}

pub struct TextDetectionPredictor {
    sess: Rc<RefCell<Session>>,
    pre_processor: PreProcessor,
    post_processor: PostProcessor,
}

impl TextDetectionPredictor {
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

    pub fn predict_path<P: AsRef<Path>>(&mut self, img_path: P) -> Result<DetectResult> {
        let img = load_image(img_path)?;
        self.predict_image(&img)
    }

    pub fn predict_image(&self, img: &RgbImage) -> Result<DetectResult> {
        let pre_output = self.pre_processor.process(img)?;
        let input = pre_output.get_input_as_ndarray();
        let mut sess = self.sess.borrow_mut();

        let outputs = sess
            .run(inputs!["x" => TensorRef::from_array_view(&input).unwrap()])
            .unwrap();

        let output = outputs["fetch_name_0"].try_extract_array::<f32>().unwrap();
        let preds = output.squeeze();
        let bitmap = preds.into_dimensionality::<Ix2>()?.to_owned();
        let boxes_result = self.post_processor.process(&bitmap)?;
        let mut bboxs = Vec::new();
        let mut scores = Vec::new();

        for b in boxes_result.iter() {
            let [tl, tr, dr, dl] = b.bbox;
            let [otl, otr, odr, odl] = [
                rescale_point(&tl, pre_output.ratio_w, pre_output.ratio_h),
                rescale_point(&tr, pre_output.ratio_w, pre_output.ratio_h),
                rescale_point(&dr, pre_output.ratio_w, pre_output.ratio_h),
                rescale_point(&dl, pre_output.ratio_w, pre_output.ratio_h),
            ];
            let bbox = Quad::new(otl, otr, odr, odl);
            bboxs.push(bbox);
            scores.push(b.score);
        }
        let result = DetectResult { bboxs, scores };
        Ok(result)
    }
}

fn rescale_point(p: &Point<u32>, ratio_w: f32, ratio_h: f32) -> Point<u32> {
    Point::new(
        (p.x as f32 * ratio_w).round() as u32,
        (p.y as f32 * ratio_h).round() as u32,
    )
}
