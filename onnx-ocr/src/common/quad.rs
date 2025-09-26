use imageproc::{geometric_transformations::Projection, point::Point};

#[derive(Debug, Clone)]
pub struct Quad {
    pub tl: Point<u32>,
    pub tr: Point<u32>,
    pub dr: Point<u32>,
    pub dl: Point<u32>,
    pub width: f32,
    pub height: f32,
}

impl Quad {
    pub fn new(tl: Point<u32>, tr: Point<u32>, dr: Point<u32>, dl: Point<u32>) -> Self {
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
            (self.tl.x as f32, self.tl.y as f32),
            (self.tr.x as f32, self.tr.y as f32),
            (self.dr.x as f32, self.dr.y as f32),
            (self.dl.x as f32, self.dl.y as f32),
        ];
        Projection::from_control_points(from, to)
    }
}
