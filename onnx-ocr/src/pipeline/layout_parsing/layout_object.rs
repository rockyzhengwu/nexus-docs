use image::RgbImage;
use std::cmp::Ordering;

use crate::{doc_layout::predictor::LayoutLabel, pipeline::ocr::OcrResultItem};

#[derive(Debug, Clone)]
pub struct TextSpan {
    label: LayoutLabel,
    bbox: [f32; 4],
    text: String,
}
impl TextSpan {
    pub fn new_from_ocr(item: &OcrResultItem) -> Self {
        let label = LayoutLabel::Text;
        let bbox = item.bbox;
        let text = item.content.to_owned();
        Self { label, bbox, text }
    }
}

#[derive(Debug, Clone)]
struct TextLine {
    spans: Vec<TextSpan>,
    direction: Direction,
    region_box: [f32; 4],
    need_new_line: bool,
    width: f32,
    height: f32,
}

impl Default for TextLine {
    fn default() -> Self {
        TextLine {
            spans: Vec::new(),
            direction: Direction::Horizontal,
            region_box: [0.0; 4],
            need_new_line: false,
            width: 0.0,
            height: 0.0,
        }
    }
}

impl TextLine {
    pub fn add_span(&mut self, span: TextSpan) {
        if self.spans.is_empty() {
            self.region_box = span.bbox;
        } else {
            self.region_box[0] = self.region_box[0].min(span.bbox[0]);
            self.region_box[1] = self.region_box[1].min(span.bbox[1]);
            self.region_box[2] = self.region_box[2].max(span.bbox[2]);
            self.region_box[3] = self.region_box[3].max(span.bbox[3]);
        }
        self.width = self.region_box[2] - self.region_box[0];
        self.height = self.region_box[3] - self.region_box[1];

        self.spans.push(span);
    }
    pub fn is_same_line(&self, span: &TextSpan) -> bool {
        if self.spans.is_empty() {
            return true;
        }
        match self.direction {
            Direction::Horizontal => {
                let start = self.region_box[1].max(span.bbox[1]);
                let end = self.region_box[3].min(span.bbox[3]);
                if end < start {
                    return false;
                }
                let p = (self.region_box[3] - self.region_box[1]).min(span.bbox[3] - span.bbox[1]);
                let ratio = end - start / p;
                ratio > 0.8
            }
            Direction::Vertical => {
                let start = self.region_box[0].max(span.bbox[0]);
                let end = self.region_box[2].min(span.bbox[2]);
                if end < start {
                    return false;
                }
                let p = (self.region_box[2] - self.region_box[0]).min(span.bbox[2] - span.bbox[0]);
                let ratio = end - start / p;
                ratio > 0.8
            }
        }
    }
    pub fn get_text(&self) -> String {
        let mut s = String::new();
        for (i, span) in self.spans.iter().enumerate() {
            if i > 0 {
                s.push('\n');
            }
            s.push_str(&span.text);
        }
        s
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub struct LayoutBlock {
    pub label: LayoutLabel,
    pub bbox: [f32; 4],
    pub direction: Direction,
    pub content: String,
    pub image: Option<RgbImage>,
    pub text_line_width: f32,
    pub text_line_height: f32,
    pub child_blocks: Vec<LayoutBlock>,
    pub num_of_lines: usize,
}

impl LayoutBlock {
    pub fn new(label: LayoutLabel, bbox: [f32; 4]) -> Self {
        let direction = Direction::Horizontal;
        let content = String::new();
        let image = None;
        let text_line_width = 0.0;
        let text_line_height = 0.0;
        let child_blocks = Vec::new();
        let num_of_lines = 0_usize;
        Self {
            label,
            bbox,
            direction,
            content,
            image,
            text_line_width,
            text_line_height,
            child_blocks,
            num_of_lines,
        }
    }

    pub fn set_image(&mut self, img: RgbImage) {
        self.image = Some(img);
    }
    pub fn center(&self) -> [f32; 2] {
        let x = (self.bbox[0] + self.bbox[2]) / 2.0;
        let y = (self.bbox[1] + self.bbox[3]) / 2.0;
        [x, y]
    }

    fn calculate_text_line_direction(&self, ocr_res: &[OcrResultItem]) -> Direction {
        let num_of_item = ocr_res.len();
        let mut h = 0;
        for item in ocr_res {
            let [x1, y1, x2, y2] = item.bbox;
            let width = x2 - x1;
            let height = y2 - y1;
            if width * 1.5 > height {
                h += 1;
            }
        }
        if h as f32 >= (num_of_item as f32 * 0.5) {
            return Direction::Horizontal;
        }
        Direction::Vertical
    }

    fn group_content_to_lines(&mut self, ocr_res: &[OcrResultItem]) -> Vec<TextLine> {
        self.direction = self.calculate_text_line_direction(ocr_res);
        let mut sorted_ocr_items = ocr_res.to_vec();
        match self.direction {
            Direction::Horizontal => {
                sorted_ocr_items.sort_by(|a, b| a.bbox[1].total_cmp(&b.bbox[1]));
            }
            Direction::Vertical => {
                sorted_ocr_items.sort_by(|a, b| b.bbox[0].total_cmp(&a.bbox[0]));
            }
        }
        let mut res = Vec::new();
        if sorted_ocr_items.is_empty() {
            return res;
        }
        let mut current_line = TextLine::default();
        current_line.direction = self.direction.clone();
        for item in sorted_ocr_items.iter() {
            let span = TextSpan::new_from_ocr(item);
            if current_line.spans.is_empty() {
                current_line.add_span(span);
            } else {
                if current_line.is_same_line(&span) {
                    current_line.add_span(span);
                } else {
                    res.push(current_line.clone());
                    current_line = TextLine::default();
                    current_line.add_span(span);
                }
            }
        }
        if !current_line.spans.is_empty() {
            res.push(current_line);
        }
        let all_width: f32 = res.iter().map(|v| v.region_box[2] - v.region_box[0]).sum();
        if !res.is_empty() {
            self.text_line_width = all_width / res.len() as f32;
        }
        let all_height: f32 = res.iter().map(|v| v.region_box[3] - v.region_box[1]).sum();
        if !res.is_empty() {
            self.text_line_height = all_height / res.len() as f32;
        }
        res
    }

    pub fn update_text_content(&mut self, ocr_res: &[OcrResultItem]) {
        let lines = self.group_content_to_lines(ocr_res);
        let mut content = String::new();
        for line in lines {
            content.push_str(line.get_text().as_str());
        }
        self.content = content;
    }
    pub fn set_table_content(&mut self, content: String) {
        self.content = content;
    }
}

#[derive(Debug)]
pub struct LayoutRegion {
    blocks: Vec<LayoutBlock>,
    bbox: [f32; 4],
    text_line_width: f32,
    text_line_height: f32,
    direction: Direction,
}

impl LayoutRegion {
    pub fn new(blocks: Vec<LayoutBlock>, bbox: [f32; 4]) -> Self {
        LayoutRegion {
            blocks,
            bbox,
            text_line_width: 20.0,
            text_line_height: 10.0,
            direction: Direction::Horizontal,
        }
    }
    pub fn blocks(&self) -> &[LayoutBlock] {
        self.blocks.as_slice()
    }

    pub fn init_region_info(&mut self) {
        let mut horizone_num = 0;
        let mut text_line_widths = Vec::new();
        let mut text_line_heights = Vec::new();
        for block in self.blocks.iter() {
            if block.direction == Direction::Horizontal {
                horizone_num += 1;
            }
            if block.label == LayoutLabel::Text {
                text_line_widths.push(block.text_line_width);
                text_line_heights.push(block.text_line_height);
            }
        }
        if horizone_num as f32 >= self.blocks.len() as f32 * 0.5 {
            self.direction = Direction::Horizontal;
        } else {
            self.direction = Direction::Vertical;
        }
        self.text_line_width = if text_line_widths.is_empty() {
            20.0
        } else {
            let total: f32 = text_line_widths.iter().sum();
            total / (text_line_widths.len() as f32)
        };
        self.text_line_height = if text_line_heights.is_empty() {
            10.0
        } else {
            let total: f32 = text_line_heights.iter().sum();
            total / (text_line_heights.len() as f32)
        };
    }

    pub fn sort_blocks(&mut self) {
        self.blocks.sort_by(|a, b| {
            cmp_block(
                a,
                b,
                &self.direction,
                self.text_line_width,
                self.text_line_height,
            )
        });
    }
}

pub fn cmp_block(
    a: &LayoutBlock,
    b: &LayoutBlock,
    direction: &Direction,
    text_line_width: f32,
    text_line_height: f32,
) -> Ordering {
    if direction == &Direction::Horizontal {
        let y1 = a.bbox[1].round() as u32 / (text_line_height.round() as u32);
        let y2 = b.bbox[1].round() as u32 / (text_line_height.round() as u32);
        match y1.cmp(&y2) {
            Ordering::Equal => {
                let x1 = a.bbox[0].round() as u32 / (text_line_width.round() as u32);
                let x2 = b.bbox[0].round() as u32 / (text_line_width.round() as u32);
                match x1.cmp(&x2) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Equal => {
                        let center1 = a.center();
                        let dist1 = center1[0] * center1[0] + center1[1] * center1[1];
                        let center2 = b.center();
                        let dist2 = center2[0] * center2[0] + center2[1] * center2[1];
                        dist1.total_cmp(&dist2)
                    }
                }
            }
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        }
    } else {
        let x1 = a.bbox[0].round() as u32 / (text_line_width.round() as u32);
        let x2 = b.bbox[0].round() as u32 / (text_line_width.round() as u32);

        match x1.cmp(&x2) {
            Ordering::Equal => {
                let y1 = a.bbox[1].round() as u32 / (text_line_width.round() as u32);
                let y2 = b.bbox[1].round() as u32 / (text_line_width.round() as u32);
                match y1.cmp(&y2) {
                    Ordering::Less => Ordering::Greater,
                    Ordering::Greater => Ordering::Less,
                    Ordering::Equal => {
                        let center1 = a.center();
                        let dist1 = center1[0] * center1[0] + center1[1] * center1[1];
                        let center2 = b.center();
                        let dist2 = center2[0] * center2[0] + center2[1] * center2[1];
                        dist1.total_cmp(&dist2).reverse()
                    }
                }
            }
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
        }
    }
}
