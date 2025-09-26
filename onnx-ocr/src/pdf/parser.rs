use anyhow::Result;
use image::RgbImage;
use pdfium_render::prelude::*;

pub fn pdf_page_to_image(bytes: &[u8], page_num: u16, password: Option<&str>) -> Result<RgbImage> {
    let pdfium = Pdfium::default();
    let document = pdfium.load_pdf_from_byte_slice(bytes, password)?;
    let page = document.pages().get(page_num)?;

    let render_config = PdfRenderConfig::default().scale_page_width_by_factor(150.0 / 72.0);
    let image = page
        .render_with_config(&render_config)?
        .as_image()
        .into_rgb8();
    Ok(image)
}
