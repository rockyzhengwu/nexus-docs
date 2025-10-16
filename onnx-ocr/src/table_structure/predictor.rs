use anyhow::Result;
use image::RgbImage;
use ndarray::Ix2;
use ort::{inputs, session::Session, value::TensorRef};
use std::fs::File;
use std::io::BufReader;
use std::{cell::RefCell, path::Path, rc::Rc};

use crate::common::{imgproc::load_image, onnx::load_session};

use crate::table_structure::{
    postprocess::{PostProcessor, TableStructure},
    preprocess::PreProcessor,
};

pub struct TableStructurePredictor {
    sess: Rc<RefCell<Session>>,
    pre_processor: PreProcessor,
    post_processor: PostProcessor,
}

fn load_character<P: AsRef<Path>>(p: P) -> Result<Vec<String>> {
    let f = File::open(p)?;
    let reader = BufReader::new(f);
    let result: Vec<String> = serde_json::from_reader(reader)?;
    Ok(result)
}

impl TableStructurePredictor {
    pub fn try_new<P: AsRef<Path>>(model_path: P, character_path: P) -> Result<Self> {
        let sess = Rc::new(RefCell::new(load_session(model_path)?));
        let pre_processor = PreProcessor::default();
        let character = load_character(character_path)?;
        let post_processor = PostProcessor::new(character);
        Ok(Self {
            sess,
            pre_processor,
            post_processor,
        })
    }

    pub fn predict_path<P: AsRef<Path>>(&mut self, img_path: P) -> Result<TableStructure> {
        let img = load_image(img_path)?;
        self.predict_image(&img)
    }

    pub fn predict_image(&self, img: &RgbImage) -> Result<TableStructure> {
        let pre_output = self.pre_processor.process(img)?;
        let input = pre_output.input();
        let mut sess = self.sess.borrow_mut();

        let outputs = sess
            .run(inputs!["x" => TensorRef::from_array_view(&input).unwrap()])
            .unwrap();

        let bbox_logits = outputs["fetch_name_0"].try_extract_array::<f32>().unwrap();
        let bbox_pred = bbox_logits
            .squeeze()
            .into_dimensionality::<Ix2>()?
            .to_owned();
        let strcture_logits = outputs["fetch_name_1"].try_extract_array::<f32>().unwrap();
        let structure_pred = strcture_logits
            .squeeze()
            .into_dimensionality::<Ix2>()?
            .to_owned();

        let post_result =
            self.post_processor
                .process(&bbox_pred, &structure_pred, pre_output.ori_shape())?;
        Ok(post_result)
    }
}
