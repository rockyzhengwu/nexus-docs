use crate::doc_layout::predictor::LayoutLabel;
use crate::model_context::ModelContext;
use crate::pipeline::layout_parsing::doc_analysise::LayoutParser;
use anyhow::Result;
use image::ImageReader;
use std::io::Cursor;

pub fn image_to_markdown(buffer: &[u8]) -> Result<String> {
    let cursor = Cursor::new(buffer);
    let reader = ImageReader::new(cursor).with_guessed_format()?;
    let original_img = reader.decode()?.to_rgb8();
    let context = ModelContext::new().unwrap();
    let mut parser = LayoutParser::new(&context);
    let res = parser.parse(&original_img).unwrap();
    let markdown = res.to_markdown()?;
    return Ok(markdown);
}
