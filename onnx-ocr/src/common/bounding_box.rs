use imageproc::point::Point;

#[derive(Debug)]
pub struct BoundingBox {
    pub points: Vec<Point<u32>>,
}

impl BoundingBox {
    pub fn new(points: Vec<Point<u32>>) -> Self {
        BoundingBox { points }
    }
}
