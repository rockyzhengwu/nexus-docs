use core::f32;

use anyhow::Result;
use image::{
    RgbImage,
    imageops::{crop_imm, rotate90, rotate180, rotate270},
};
use itertools::Itertools;

use crate::{
    doc_text_ori::predictor::RotateAngle,
    model_context::ModelContext,
    pipeline::ocr::{self, OcrResultItem},
    table_cell_detection::predictor::TableCelltResult,
    table_cls::predictor::TableType,
};

#[derive(Debug, Clone)]
pub struct TableCell {
    pub coordinate: [f32; 4],
    pub score: f32,
    pub content: String,
    pub row: u32,
    pub col: u32,
    pub col_span: u32,
    pub row_span: u32,
}

impl TableCell {
    pub fn new(coordinate: [f32; 4], score: f32, content: String) -> Self {
        TableCell {
            coordinate,
            score,
            content,
            row: 0,
            col: 0,
            col_span: 1,
            row_span: 1,
        }
    }
}

pub fn extract_table(
    context: &ModelContext,
    img: &RgbImage,
    ocr_res: &[OcrResultItem],
) -> Result<TableResult> {
    let doc_text_ori_predictor = &context.doc_text_ori_predictor;
    let doc_angle = doc_text_ori_predictor.predict_image(img)?;
    let mut ocr_res = ocr_res.to_owned();
    let mut pre_img = img.to_owned();
    match doc_angle {
        RotateAngle::R0 => {}
        RotateAngle::R90 => {
            pre_img = rotate270(img);
            ocr_res = ocr::ocr(context, &pre_img)?;
        }
        RotateAngle::R180 => {
            pre_img = rotate180(img);
            ocr_res = ocr::ocr(context, &pre_img)?;
        }
        RotateAngle::R270 => {
            pre_img = rotate90(img);
            ocr_res = ocr::ocr(context, &pre_img)?;
        }
    }

    let table_cls_predictor = &context.table_cls_predictor;
    let table_type = table_cls_predictor.predict_image(&pre_img)?;
    let table_cells_result = match table_type {
        TableType::Wired => {
            let predictor = &context.wired_table_cell_predictor;
            predictor.predict_image(&pre_img)?
        }
        TableType::Wireless => {
            let predictor = &context.wireless_table_cell_predictor;
            predictor.predict_image(&pre_img)?
        }
    };
    let table_cells_result = cells_det_result_nms(table_cells_result)?;

    // 统计有多少行，多少列,
    // 每行的高度，每列的宽度
    // 根据上面信息构造 table
    // TODO match ocr result with table cell, instead of ocr every table cell
    for item in ocr_res.iter() {
        println!("{}", item.content);
    }
    let mut table_cells = Vec::new();
    for cell in table_cells_result.iter() {
        let ocr_items = match_ocr_items(&cell.coordinate, ocr_res.as_slice());
        let mut content = String::new();
        for item in ocr_items.iter() {
            content.push_str(item.content.as_str());
        }
        // let cell_ocr_items =
        //let cell_img = crop_imm(img, x1.ceil() as u32, y1.ceil() as u32, width, height).to_image();
        //let cell_ocr_reslt = ocr::ocr(context, &cell_img)?;
        //et content = cell_ocr_reslt.iter().map(|v| &v.content).join("\n");
        let tcell = TableCell::new(cell.coordinate, cell.score, content);
        table_cells.push(tcell);
    }
    if table_cells.is_empty() {
        return Ok(TableResult {
            cells: table_cells,
            row_count: 0,
            col_count: 0,
        });
    }

    // TODO update cell row id an col_span

    table_cells.sort_by(|a, b| a.coordinate[0].total_cmp(&b.coordinate[0]));
    let all_x: Vec<f32> = table_cells.iter().map(|c| c.coordinate[0]).collect();
    let last_x = table_cells.last().unwrap().coordinate[2];
    let (col_ids, col_width) = calc_table_info(all_x, last_x);
    for (i, cell) in table_cells.iter_mut().enumerate() {
        let x = cell.coordinate[0];
        let max_x = cell.coordinate[2];
        let cell_w = max_x - x;
        let col_id = col_ids[i];
        let mut total_w = col_width[col_id as usize];
        let mut col_span = 1;
        while (total_w - cell_w).abs() > 10.0 && (x + total_w) < max_x {
            total_w += col_width[(col_id + col_span) as usize];
            col_span += 1;
        }
        cell.col = col_id;
        if col_span > 1 {
            cell.col_span = col_span
        }
    }

    table_cells.sort_by(|a, b| a.coordinate[1].total_cmp(&b.coordinate[1]));
    let all_y: Vec<f32> = table_cells.iter().map(|c| c.coordinate[1]).collect();
    let last_y = table_cells.last().unwrap().coordinate[3];
    let (row_ids, row_height) = calc_table_info(all_y, last_y);
    for (i, cell) in table_cells.iter_mut().enumerate() {
        let y = cell.coordinate[1];
        let max_y = cell.coordinate[3];
        let cell_h = max_y - y;
        let row_id = row_ids[i];
        let mut total_h = row_height[row_id as usize];
        let mut row_span = 1;
        while (total_h - cell_h).abs() > 10.0 && (y + total_h) < max_y {
            total_h += row_height[(row_id + row_span) as usize];
            row_span += 1;
        }
        cell.row = row_id;
        if row_span > 1 {
            cell.row_span = row_span;
        }
    }
    table_cells.sort_by(|a, b| {
        if a.row != b.row {
            a.row.cmp(&b.row)
        } else {
            a.col.cmp(&b.col)
        }
    });

    let table = TableResult {
        cells: table_cells,
        row_count: row_height.len() as u32,
        col_count: col_width.len() as u32,
    };

    Ok(table)
}

#[derive(Debug)]
pub struct TableResult {
    cells: Vec<TableCell>,
    col_count: u32,
    row_count: u32,
}

impl TableResult {
    pub fn to_html(&self) -> String {
        if self.cells.is_empty() {
            return String::new();
        }
        let mut html = "<table><tbody><tr>".to_string();
        let mut current_row = 0;
        for cell in self.cells.iter() {
            if cell.row != current_row {
                current_row = cell.row;
                html.push_str("</tr><tr>");
                let td = self.format_cell(cell);
                html.push_str(td.as_str());
            } else {
                let td = self.format_cell(cell);
                html.push_str(td.as_str());
            }
        }
        html.push_str("</tr></tbody></table>");
        html
    }
    pub fn format_cell(&self, cell: &TableCell) -> String {
        if cell.row_span > 1 {
            if cell.col_span > 1 {
                format!(
                    "<td colspan=\"{}\" rowspan=\"{}\">{}</td>",
                    cell.col_span, cell.row_span, cell.content
                )
            } else {
                format!("<td rowspan=\"{}\">{}</td>", cell.row_span, cell.content)
            }
        } else {
            if cell.col_span > 1 {
                format!("<td colspan=\"{}\">{}</td>", cell.col_span, cell.content)
            } else {
                format!("<td>{}</td>", cell.content)
            }
        }
    }
}

fn calc_table_info(all_value: Vec<f32>, last_value: f32) -> (Vec<u32>, Vec<f32>) {
    let cell_num = all_value.len();

    let mut row_indexs: Vec<u32> = Vec::with_capacity(cell_num);
    let mut row_height: Vec<f32> = Vec::with_capacity(cell_num);

    let mut current_y: f32 = 0.0;
    let mut current_row: u32 = 0;
    let mut tmp_y = Vec::new();
    for (i, y) in all_value.iter().enumerate() {
        if i == 0 {
            current_y = *y;
            row_indexs.push(current_row);
            tmp_y.push(current_y);
        } else {
            if (y - current_y).abs() < 10.0 {
                row_indexs.push(current_row);
                current_y = *y;
                tmp_y.push(current_y);
            } else {
                let min_y = tmp_y.iter().fold(f32::INFINITY, |a, b| a.min(*b));
                let height = y - min_y;
                tmp_y.clear();
                row_height.push(height);

                current_y = *y;
                current_row += 1;
                row_indexs.push(current_row);
                tmp_y.push(current_y);
            }
        }
    }
    if !tmp_y.is_empty() {
        current_row += 1;
        row_indexs.push(current_row);
        let height = last_value - tmp_y.iter().fold(f32::INFINITY, |a, b| a.min(*b));
        row_height.push(height);
    }
    (row_indexs, row_height)
}

fn cells_det_result_nms(cells_det_results: Vec<TableCelltResult>) -> Result<Vec<TableCelltResult>> {
    let mut sorted_results = cells_det_results;
    sorted_results.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut nms_result = Vec::new();
    let iou_threshold = 0.3;
    while !sorted_results.is_empty() {
        let first = sorted_results.pop().unwrap();
        sorted_results.retain(|result| {
            let iou = compute_iou(&first.coordinate, &result.coordinate);
            iou <= iou_threshold
        });
        nms_result.push(first);
    }
    return Ok(nms_result);
}

fn compute_iou(box1: &[f32; 4], box2: &[f32; 4]) -> f32 {
    let x_left = box1[0].max(box2[0]);
    let y_top = box1[1].max(box2[1]);
    let x_right = box1[2].min(box2[2]);
    let y_bottom = box1[3].min(box2[3]);
    if x_left > x_right || y_bottom < y_top {
        return 0.0;
    }
    let intersection_area = (x_right - x_left) * (y_bottom - y_top);
    let area_box1 = (box1[2] - box1[0]) * (box1[3] - box1[1]);
    let area_box2 = (box2[2] - box2[0]) * (box2[3] - box2[1]);
    let iou = intersection_area / (area_box1 + area_box2 - intersection_area);
    iou
}

fn match_ocr_items(bbox: &[f32; 4], ocr_res: &[OcrResultItem]) -> Vec<OcrResultItem> {
    let mut res = Vec::new();
    let [tx1, ty1, tx2, ty2] = bbox;
    for item in ocr_res.iter() {
        let [ox1, oy1, ox2, oy2] = &item.bbox;
        let x1 = ox1.max(*tx1);
        let y1 = oy1.max(*ty1);
        let x2 = ox2.min(*tx2);
        let y2 = oy2.min(*ty2);
        if x2 <= x1 || y2 <= y1 {
            continue;
        }
        if ((x2 - x1) > 3.0) & ((y2 - y1) > 3.0) {
            res.push(item.to_owned());
        }
    }
    res
}
