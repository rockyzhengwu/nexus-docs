use imageproc::point::Point;

#[derive(Debug)]
pub struct MinAreaRect {
    pub center: Point<f32>,
    /// The width of the rectangle.
    pub width: f32,
    /// The height of the rectangle.
    pub height: f32,
    /// The rotation angle of the rectangle in degrees.
    pub angle: f32,
}

impl MinAreaRect {
    pub fn new(center: Point<f32>, width: f32, height: f32, angle: f32) -> Self {
        MinAreaRect {
            center,
            width,
            height,
            angle,
        }
    }
}

impl Default for MinAreaRect {
    fn default() -> Self {
        MinAreaRect {
            center: Point::new(0.0, 0.0),
            width: 0.0,
            height: 0.0,
            angle: 0.0,
        }
    }
}
