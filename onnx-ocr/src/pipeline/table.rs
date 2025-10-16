use anyhow::Result;
use image::{RgbImage, imageops::crop_imm};
use itertools::cons_tuples;

use crate::{
    doc_layout::predictor, model_context::ModelContext, pipeline::ocr,
    table_cls::predictor::TableType, table_structure,
};

#[derive(Debug, Clone)]
pub struct TableCell {
    pub coordinate: [f32; 4],
    pub score: f32,
    pub content: String,
}

impl TableCell {
    pub fn new(coordinate: [f32; 4], score: f32, content: String) -> Self {
        TableCell {
            coordinate,
            score,
            content,
        }
    }
}

pub fn extract_table(context: &ModelContext, img: &RgbImage) -> Result<Table> {
    let table_cls_predictor = &context.table_cls_predictor;
    let table_type = table_cls_predictor.predict_image(img)?;
    let table_cells = match table_type {
        TableType::Wired => {
            let predictor = &context.wired_table_cell_predictor;
            predictor.predict_image(img)?
        }
        TableType::Wireless => {
            let predictor = &context.wireless_table_cell_predictor;
            predictor.predict_image(img)?
        }
    };
    let table_structure = match table_type {
        TableType::Wired => {
            let predictor = &context.wired_table_structure_predictor;
            predictor.predict_image(img)?
        }
        TableType::Wireless => {
            let predictor = &context.wireless_table_structure_predictor;
            predictor.predict_image(img)?
        }
    };
    println!("table:{:?}", table_structure);

    // TODO  threshold
    // TODO nms table_cells

    let mut images = Vec::new();
    let mut i = 0;
    let mut cells = Vec::new();
    for bbox in table_cells.iter() {
        let [x1, y1, x2, y2] = bbox.coordinate;
        let x = x1.min(x2).round() as u32;
        let y = y1.min(y2).round() as u32;
        let height = (y2 - y1).abs().round() as u32;
        let width = (x2 - x1).abs().round() as u32;
        let cell_img = crop_imm(img, x, y, width, height).to_image();
        cell_img.save(format!("cell_image{}.jpg", i)).unwrap();
        let ocr_result = ocr::ocr(context, &cell_img)?;
        let mut content = String::new();
        for item in ocr_result.iter() {
            content.push_str(item.content.as_str());
        }
        images.push(cell_img);
        i += 1;
        let cell = TableCell::new([x1, y1, x2, y2], bbox.score, content);
        cells.push(cell);
    }
    let table = Table::new(cells);
    Ok(table)
}

#[derive(Debug)]
pub struct Table {
    cells: Vec<TableCell>,
}

impl Table {
    pub fn new(cells: Vec<TableCell>) -> Self {
        Table { cells }
    }
    pub fn to_html(&self) -> String {
        // TODO 统计有多少行，多少列，行高和列高
        let mut sorted_cells = self.cells.clone();

        return "".to_string();
    }
    // TODO create html
}
