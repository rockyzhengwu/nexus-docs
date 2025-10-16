use crate::model_context::ModelContext;
use anyhow::Result;
use image::RgbImage;

pub fn doc_analysise(context: &ModelContext, img: &RgbImage) -> Result<()> {
    let layout_predictor = &context.layout_predictor;
    let layout_result = layout_predictor.predict_image(&img)?;
    // TODO merge and delete some boxes
    for obj in layout_result.iter() {
        match obj.label.as_str() {
            "paragraph_title" | "text" | "abstract" | "figure_title" | "reference"
            | "doc_title" | "footnote" | "header" | "footer" | "aside_text" => {}
            "image" => {}
            "table" => {}
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
