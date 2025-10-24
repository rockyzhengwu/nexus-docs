use crate::{
    common::{imgproc::load_image, onnx::load_session},
    doc_text_ori::preprocess::PreProcessor,
};
use anyhow::Result;
use image::RgbImage;
use ndarray::Array4;
use ort::{inputs, session::Session, value::TensorRef};
use std::{cell::RefCell, path::Path, rc::Rc};

pub struct DocTextOriPredictor {
    sess: Rc<RefCell<Session>>,
    pre_processor: PreProcessor,
}

#[derive(Debug)]
pub enum RotateAngle {
    R0,
    R90,
    R180,
    R270,
}

impl DocTextOriPredictor {
    pub fn try_new<P: AsRef<Path>>(model_path: P) -> Result<Self> {
        let sess = Rc::new(RefCell::new(load_session(model_path)?));
        let pre_processor = PreProcessor::default();
        Ok(Self {
            sess,
            pre_processor,
        })
    }

    pub fn predict_path<P: AsRef<Path>>(&mut self, img_path: P) -> Result<RotateAngle> {
        let img = load_image(img_path)?;
        self.predict_image(&img)
    }

    pub fn predict_image(&self, img: &RgbImage) -> Result<RotateAngle> {
        let input_img = self.pre_processor.process(&img);

        let input_width = input_img.width();
        let input_height = input_img.height();
        let mut input = Array4::<f32>::zeros((1, 3, input_height as usize, input_width as usize));
        for y in 0..input_height as usize {
            for x in 0..input_width as usize {
                let pixel = input_img.get_pixel(x as u32, y as u32);
                let [r, g, b] = pixel.0;
                input[[0, 0, y, x]] = r;
                input[[0, 1, y, x]] = g;
                input[[0, 2, y, x]] = b;
            }
        }
        let mut sess = self.sess.borrow_mut();
        let outputs = sess
            .run(inputs![
                "x" =>TensorRef::from_array_view(&input).unwrap(),
            ])
            .unwrap();

        let output = outputs["fetch_name_0"].try_extract_array::<f32>()?;
        let scores = output.squeeze();
        let mut argmax = 0;
        let mut max_score = 0.0;
        for (index, score) in scores.iter().enumerate() {
            if score > &max_score {
                max_score = score.to_owned();
                argmax = index;
            }
        }
        if argmax == 0 {
            return Ok(RotateAngle::R0);
        }
        if argmax == 1 {
            return Ok(RotateAngle::R90);
        }
        if argmax == 2 {
            return Ok(RotateAngle::R180);
        }
        if argmax == 3 {
            return Ok(RotateAngle::R270);
        }
        Ok(RotateAngle::R0)
    }
}
