use std::collections::HashMap;

use crate::common::quad::Quad;
use crate::doc_layout::predictor::{LayoutLabel, LayoutResult};
use crate::model_context::ModelContext;
use crate::pipeline::layout_parsing::layout_object::{LayoutBlock, LayoutRegion, TextSpan};
use crate::pipeline::layout_parsing::util::caculate_overlap_ratio;
use crate::pipeline::ocr::{self, OcrResultItem};
use crate::pipeline::table::extract_table;

use anyhow::Result;
use image::RgbImage;
use image::imageops::{crop_imm, rotate90};

#[derive(Debug)]
pub struct Document {
    pub objects: Vec<LayoutBlock>,
}

#[derive(Debug, Default)]
struct ParsingInfo {
    block_to_ocr: HashMap<usize, Vec<usize>>,
    ocr_to_block: HashMap<usize, Vec<usize>>,
    region_box: [f32; 4],
    max_block_area: f32,
}

pub struct LayoutParser<'a> {
    context: &'a ModelContext,
}

impl<'a> LayoutParser<'a> {
    pub fn new(context: &'a ModelContext) -> Self {
        LayoutParser { context }
    }

    pub fn parse(&mut self, img: &RgbImage) -> Result<LayoutRegion> {
        let layout_predictor = &self.context.layout_predictor;
        let layout_result = layout_predictor.predict_image(&img)?;
        let mut all_ocr_res = ocr::ocr(self.context, img)?;
        // sandardized_layout
        let mut layout_result = remove_overlap_block(layout_result.as_slice(), 0.6);
        let parsing_info = self.match_block_and_ocr(img, &mut layout_result, &mut all_ocr_res)?;

        let mut region = self.parsing_layout_region(
            img,
            layout_result.as_slice(),
            &all_ocr_res.as_slice(),
            &parsing_info,
        )?;
        region.sort_blocks();
        Ok(region)
    }

    fn match_block_and_ocr(
        &self,
        img: &RgbImage,
        layout_res: &mut Vec<LayoutResult>,
        ocr_res: &mut Vec<OcrResultItem>,
    ) -> Result<ParsingInfo> {
        let mut region_box: [f32; 4] = [0.0; 4];
        let mut doc_title_num = 0;
        let mut max_y: f32 = 0.0;
        let mut block_to_ocr: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut ocr_to_block: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut max_block_area: f32 = 0.0;
        let mut footnotes = Vec::new();
        let mut paragraph_titles = Vec::new();
        for (obj_id, obj) in layout_res.iter().enumerate() {
            let [_, _, _, block_y2] = obj.coordinate;
            update_region_box(&mut region_box, &obj.coordinate);
            max_block_area = max_block_area.max(obj.area());
            if obj.label == LayoutLabel::Footnote {
                footnotes.push(obj_id);
            } else if obj.label == LayoutLabel::ParaGraphTitle {
                paragraph_titles.push(obj_id);
            } else if obj.label == LayoutLabel::DocTitle {
                doc_title_num += 1;
            } else if obj.label == LayoutLabel::Text {
                max_y = max_y.max(block_y2);
            }

            if obj.label != LayoutLabel::Formula && obj.label != LayoutLabel::Seal {
                let matched_ocr = get_sub_region_ocr_res(ocr_res, &[obj]);
                for (_, ocr_id) in matched_ocr.iter().enumerate() {
                    if ocr_to_block.contains_key(&ocr_id) {
                        ocr_to_block.get_mut(&ocr_id).unwrap().push(obj_id);
                    } else {
                        ocr_to_block.insert(ocr_id.to_owned(), vec![obj_id]);
                    }
                }
                block_to_ocr.insert(obj_id, matched_ocr);
            }
        }

        // fix footenotes which y position is up on max_y
        for foot_id in footnotes.iter() {
            let block = layout_res.get_mut(foot_id.to_owned()).unwrap();
            if block.coordinate[3] < max_y {
                block.label = LayoutLabel::new_from_str("text");
            }
        }

        // fix doc_title: if there has no doc_title and only on paragraph_title and it's area bigger than 0.3 *
        // max_block_area change it's to doc_title
        if doc_title_num == 0 && paragraph_titles.len() == 1 {
            let ptitle = layout_res.get_mut(paragraph_titles[0]).unwrap();
            let ptitile_area = ptitle.area();
            if ptitile_area > max_block_area * 0.3 {
                ptitle.label = LayoutLabel::new_from_str("doc_title");
            }
        }
        // on ocr maped to mutil block
        for (ocr_id, block_ids) in ocr_to_block.iter() {
            if block_ids.len() > 1 {
                let ocr_bbox = ocr_res[ocr_id.to_owned()].bbox;
                let mut match_num = 0;
                for block_id in block_ids.iter() {
                    let block = layout_res.get(block_id.to_owned()).unwrap();
                    let crop_bbox = get_bbox_intersection(&ocr_bbox, &block.coordinate);
                    match crop_bbox {
                        Some(bbox) => {
                            let mut sub_img = crop_sub_img(&bbox, img);
                            // TODO fix this
                            if sub_img.width() < sub_img.height() {
                                sub_img = rotate90(&sub_img);
                            }
                            let text_rec_res =
                                self.context.text_rec_predictor.predict(vec![sub_img])?;
                            let text = &text_rec_res[0];
                            if text.1 >= 0.5 {
                                if match_num == 0 {
                                    ocr_res[ocr_id.to_owned()].content = text.0.to_owned();
                                } else {
                                    let content = text.0.to_string();
                                    let polys = Quad::new_from_bbox(&bbox);
                                    let new_ocr = OcrResultItem::new(polys, content, bbox);
                                    ocr_res.push(new_ocr);
                                    block_to_ocr
                                        .get_mut(block_id)
                                        .unwrap()
                                        .retain(|v| v != ocr_id);
                                    block_to_ocr
                                        .get_mut(block_id)
                                        .unwrap()
                                        .push(ocr_res.len() - 1);
                                }

                                match_num += 1;
                            }
                        }
                        None => {
                            continue;
                        }
                    }
                }
            }
        }
        let parseing_info = ParsingInfo {
            block_to_ocr,
            ocr_to_block,
            region_box,
            max_block_area,
        };
        Ok(parseing_info)
    }

    fn parsing_layout_region(
        &self,
        img: &RgbImage,
        layout_res: &[LayoutResult],
        ocr_res: &[OcrResultItem],
        parsing_info: &ParsingInfo,
    ) -> Result<LayoutRegion> {
        let mut doc_objects = Vec::new();
        for (obj_id, obj) in layout_res.iter().enumerate() {
            let mut block = LayoutBlock::new(obj.label.clone(), obj.coordinate);
            match obj.label {
                LayoutLabel::ParaGraphTitle
                | LayoutLabel::Text
                | LayoutLabel::Abstract
                | LayoutLabel::FigureTitle
                | LayoutLabel::Reference
                | LayoutLabel::DocTitle
                | LayoutLabel::Footnote
                | LayoutLabel::Header
                | LayoutLabel::Number
                | LayoutLabel::Footer
                | LayoutLabel::Content
                | LayoutLabel::Algorithm
                | LayoutLabel::ReferenceContent
                | LayoutLabel::AsideText => {
                    if let Some(ocr_ids) = parsing_info.block_to_ocr.get(&obj_id) {
                        let items: Vec<OcrResultItem> = ocr_ids
                            .iter()
                            .map(|ocr_id| ocr_res.get(ocr_id.to_owned()).unwrap().to_owned())
                            .collect();
                        block.update_text_content(items.as_slice());
                    }
                }
                LayoutLabel::Chart => {
                    println!("chart")
                }

                LayoutLabel::Formula | LayoutLabel::FormulaNumber => {
                    println!("formular")
                }

                LayoutLabel::Image => {
                    let img = crop_sub_img(&obj.coordinate, img);
                    block.set_image(img);
                }
                LayoutLabel::Table => {
                    if let Some(ocr_ids) = parsing_info.block_to_ocr.get(&obj_id) {
                        let ocr_items: Vec<OcrResultItem> = ocr_ids
                            .iter()
                            .map(|ocr_id| ocr_res[ocr_id.to_owned()].to_owned())
                            .collect();
                        let table_img = crop_sub_img(&obj.coordinate, img);
                        let table = extract_table(self.context, &table_img, ocr_items.as_slice())?;
                        block.set_table_content(table.to_html());
                    } else {
                        println!("table ocr is None");
                    }
                }
                LayoutLabel::Seal => {
                    println!("ignore seal");
                }
            }
            doc_objects.push(block);
        }
        let region = LayoutRegion::new(doc_objects, parsing_info.region_box);
        Ok(region)
    }
}

fn remove_overlap_block(layout_res: &[LayoutResult], threshod: f32) -> Vec<LayoutResult> {
    let n = layout_res.len();
    let mut removed_boxes = Vec::new();
    for i in 0..n {
        let current = &layout_res[i];
        let current_area = current.area();
        if removed_boxes.contains(&i) {
            continue;
        }

        for j in (i + 1)..n {
            if removed_boxes.contains(&j) {
                continue;
            }
            let other = &layout_res[j];
            let other_area = other.area();
            let overlap_ratio = caculate_overlap_ratio(
                &current.coordinate,
                current_area,
                &other.coordinate,
                other_area,
            );
            if overlap_ratio > threshod {
                if current.label == LayoutLabel::Image && other.label != LayoutLabel::Image {
                    removed_boxes.push(i);
                } else if current.label != LayoutLabel::Image && other.label == LayoutLabel::Image {
                    removed_boxes.push(j);
                } else {
                    if current_area < other_area {
                        removed_boxes.push(i);
                    } else {
                        removed_boxes.push(j);
                    }
                }
            }
        }
    }
    let mut res = Vec::new();
    for (i, obj) in layout_res.iter().enumerate() {
        if removed_boxes.contains(&i) {
            continue;
        }
        res.push(obj.to_owned());
    }
    res
}

fn update_region_box(region: &mut [f32; 4], rect: &[f32; 4]) {
    let [rx1, ry1, rx2, ry2] = rect;
    region[0] = region[0].min(*rx1);
    region[1] = region[1].min(*ry1);
    region[2] = region[2].max(*rx2);
    region[2] = region[2].max(*ry2);
}

fn get_sub_region_ocr_res(ocr_res: &[OcrResultItem], sub_regions: &[&LayoutResult]) -> Vec<usize> {
    let mut matched_idx = Vec::new();
    for region in sub_regions.iter() {
        let [rx1, ry1, rx2, ry2] = region.coordinate;
        for (i, ocr) in ocr_res.iter().enumerate() {
            if matched_idx.contains(&i) {
                continue;
            }
            let [ox1, oy1, ox2, oy2] = ocr.bbox;
            let x1 = ox1.max(rx1);
            let y1 = oy1.max(ry1);
            let x2 = ox2.min(rx2);
            let y2 = oy2.min(ry2);
            if ((x2 - x1) > 3.0_f32) & ((y2 - y1) > 3.0_f32) {
                matched_idx.push(i);
            }
        }
    }
    matched_idx
}

fn get_bbox_intersection(rect1: &[f32; 4], rect2: &[f32; 4]) -> Option<[f32; 4]> {
    let x1 = rect1[0].max(rect2[0]);
    let y1 = rect1[1].max(rect2[1]);
    let x2 = rect1[2].min(rect2[2]);
    let y2 = rect1[3].min(rect2[3]);
    if x2 <= x1 || y2 <= y1 {
        return None;
    }
    Some([x1, y1, x2, y2])
}

fn caculate_rect_area(rect: &[f32; 4]) -> f32 {
    let [x1, y1, x2, y2] = rect;
    let width = x2 - x1;
    let height = y2 - y1;
    width * height
}

fn crop_sub_img(bbox: &[f32; 4], img: &RgbImage) -> RgbImage {
    let [x1, y1, x2, y2] = bbox;
    let x = x1.ceil() as u32;
    let y = y1.ceil() as u32;
    let width = (x2 - x1).ceil() as u32;
    let height = (y2 - y1).ceil() as u32;
    let sub_img = crop_imm(img, x, y, width, height).to_image();
    sub_img
}
