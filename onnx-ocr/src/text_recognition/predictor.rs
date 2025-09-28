use anyhow::Result;
use image::RgbImage;
use ndarray::{Array4, Ix2};
use ort::{inputs, session::Session, value::TensorRef};
use std::{cell::RefCell, collections::HashMap, fs::File, io::BufReader, path::Path, rc::Rc};

use crate::{
    common::onnx::load_session,
    text_recognition::{postprocess::PostProcessor, preprocess::PreProcessor},
};

pub struct TextRecognitionPredictor {
    sess: Rc<RefCell<Session>>,
    character_dict: HashMap<u32, String>,
    pre_processor: PreProcessor,
    post_processor: PostProcessor,
}

fn load_character_dict<P: AsRef<Path>>(path: P) -> Result<HashMap<u32, String>> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let result: HashMap<u32, String> = serde_json::from_reader(reader)?;
    return Ok(result);
}

impl TextRecognitionPredictor {
    pub fn try_new<P: AsRef<Path>>(model_path: P, character_path: P) -> Result<Self> {
        let sess = Rc::new(RefCell::new(load_session(model_path)?));
        let pre_processor = PreProcessor::default();
        let post_processor = PostProcessor::default();
        let character_dict = load_character_dict(character_path)?;
        Ok(Self {
            sess,
            pre_processor,
            post_processor,
            character_dict,
        })
    }

    pub fn predict(&self, images: Vec<RgbImage>) -> Result<Vec<(String, f32)>> {
        let mut predicted_text = Vec::new();
        for img in images {
            let input_img = self.pre_processor.process(&img);
            let height = input_img.height();
            let width = input_img.width();

            let mut input = Array4::<f32>::zeros((1, 3, height as usize, width as usize));

            for y in 0..height as usize {
                for x in 0..width as usize {
                    let pixel = input_img.get_pixel(x as u32, y as u32);
                    let [r, g, b] = pixel.0;
                    input[[0, 0, y, x]] = b;
                    input[[0, 1, y, x]] = g;
                    input[[0, 2, y, x]] = r;
                }
            }
            let mut sess = self.sess.borrow_mut();
            let outputs = sess
                .run(inputs!["x" => TensorRef::from_array_view(&input).unwrap()])
                .unwrap();
            let output = outputs["fetch_name_0"].try_extract_array::<f32>().unwrap();
            let preds = output.squeeze();
            let preds = preds.into_dimensionality::<Ix2>()?.to_owned();
            let idx_score = self.post_processor.process(&preds)?;
            let mut content = String::new();
            let mut max_score = 0.0;
            for (id, score) in idx_score.iter() {
                if *id == 0 {
                    continue;
                }
                if score > &max_score {
                    max_score = score.to_owned()
                }
                let s = self.character_dict.get(id).unwrap();
                content.push_str(s.as_str())
            }
            predicted_text.push((content, max_score));
        }
        Ok(predicted_text)
    }
}
