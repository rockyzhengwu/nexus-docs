use crate::model_context::ModelContext;
use crate::pipeline::ocr::OcrResultItem;
use crate::pipeline::table::{Table, extract_table};
use anyhow::Result;
use image::RgbImage;
use image::imageops::crop_imm;

pub enum LayoutObject {}

pub struct ImageObj {
    img: RgbImage,
    coordinate: [f32; 4],
}

pub struct TableObj {
    table: Table,
    coordinate: [f32; 4],
}

pub struct TextObj {
    content: Vec<OcrResultItem>,
    coordinate: [f32; 4],
}

pub fn doc_analysise(context: &ModelContext, img: &RgbImage) -> Result<()> {
    let layout_predictor = &context.layout_predictor;
    let layout_result = layout_predictor.predict_image(&img)?;
    // TODO merge and delete some boxes
    for obj in layout_result.iter() {
        match obj.label.as_str() {
            "paragraph_title" | "text" | "abstract" | "figure_title" | "reference"
            | "doc_title" | "footnote" | "header" | "footer" | "aside_text" => {}
            "image" => {}
            "table" => {
                let [x1, y1, x2, y2] = obj.coordinate;
                let x = x1.ceil() as u32;
                let y = y1.ceil() as u32;
                let width = (x2 - x1).ceil() as u32;
                let height = (y2 - y1).ceil() as u32;
                let table_img = crop_imm(img, x, y, width, height).to_image();
                let table = extract_table(context, &table_img)?;
            }
            "seal" => {
                println!("ignore seal");
            }
            _ => {
                println!("{}", obj.label);
            }
        }
        println!("layout_obj:{:?}", obj);
    }

    unimplemented!()
}
