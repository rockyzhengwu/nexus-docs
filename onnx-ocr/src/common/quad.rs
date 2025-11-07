use core::f32;

use imageproc::{geometric_transformations::Projection, point::Point};

#[derive(Debug, Clone)]
pub struct Quad {
    pub tl: Point<f32>,
    pub tr: Point<f32>,
    pub dr: Point<f32>,
    pub dl: Point<f32>,
    pub width: f32,
    pub height: f32,
}

impl Quad {
    pub fn new_from_bbox(bbox: &[f32; 4]) -> Self {
        let [x1, y1, x2, y2] = bbox.to_owned();
        let tl = Point::new(x1, y1);
        let tr = Point::new(x2, y1);
        let dr = Point::new(x2, y2);
        let dl = Point::new(x1, y2);
        let width = x2 - x1;
        let height = y2 - y1;
        Self {
            tl,
            tr,
            dr,
            dl,
            width,
            height,
        }
    }
    pub fn new(tl: Point<f32>, tr: Point<f32>, dr: Point<f32>, dl: Point<f32>) -> Self {
        let dx = tr.x as f32 - tl.x as f32;
        let dy = tr.y as f32 - tl.y as f32;
        let width = ((dx * dx) + (dy * dy)).sqrt();

        let dx = dl.x as f32 - tl.x as f32;
        let dy = dl.y as f32 - tl.y as f32;
        let height = ((dx * dx) + (dy * dy)).sqrt();
        Quad {
            tl,
            tr,
            dr,
            dl,
            width,
            height,
        }
    }
    pub fn projection(&self) -> Option<Projection> {
        let to = [
            (0.0, 0.0),
            (self.width, 0.0),
            (self.width, self.height),
            (0.0, self.height),
        ];
        let from = [
            (self.tl.x, self.tl.y),
            (self.tr.x, self.tr.y),
            (self.dr.x, self.dr.y),
            (self.dl.x, self.dl.y),
        ];
        Projection::from_control_points(from, to)
    }
    pub fn bbox(&self) -> [f32; 4] {
        let x1 = [self.tl.x, self.tr.x, self.dr.x, self.dl.x]
            .iter()
            .fold(f32::MAX, |a: f32, &b| a.min(b));
        let y1 = [self.tl.y, self.tr.y, self.dr.y, self.dl.y]
            .iter()
            .fold(f32::MAX, |a: f32, &b| a.min(b));

        let x2 = [self.tl.x, self.tr.x, self.dr.x, self.dl.x]
            .iter()
            .fold(0.0, |a: f32, &b| a.max(b));
        let y2 = [self.tl.y, self.tr.y, self.dr.y, self.dl.y]
            .iter()
            .fold(0.0_f32, |a, &b| a.max(b));
        [x1, y1, x2, y2]
    }
}
