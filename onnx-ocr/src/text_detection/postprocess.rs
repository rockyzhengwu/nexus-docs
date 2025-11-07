use anyhow::Result;
use clipper2_sys::{ClipperOffset, Path64, Point64};
use image::{GrayImage, ImageBuffer, Luma};
use imageproc::{
    contours::find_contours, drawing::draw_polygon_mut, geometry::min_area_rect, point::Point,
};
use ndarray::Array2;

pub struct PostProcessor {
    pub threshold: f32,
    pub box_threshold: f32,
    pub max_candidates: usize,
    pub unclip_ratio: f32,
    pub min_size: f32,
}

impl Default for PostProcessor {
    fn default() -> Self {
        PostProcessor {
            threshold: 0.3,
            box_threshold: 0.6,
            max_candidates: 1000,
            unclip_ratio: 1.3,
            min_size: 3.0,
        }
    }
}
pub struct BoxResult {
    pub bbox: [Point<i32>; 4],
    pub score: f32,
}

impl PostProcessor {
    pub fn process(&self, pred: &Array2<f32>) -> Result<Vec<BoxResult>> {
        let binary = pred.mapv(|x| if x >= self.threshold { 255_u8 } else { 0 });
        let (height, width) = binary.dim();

        let contiguous_arr = binary.as_slice().unwrap();
        let pixels: Vec<u8> = contiguous_arr.to_vec();

        let gray_img: ImageBuffer<Luma<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width as u32, height as u32, pixels).unwrap();

        let contours = find_contours::<i32>(&gray_img);

        let num_contour = std::cmp::min(contours.len(), self.max_candidates);
        let mut results = Vec::new();
        for i in 0..num_contour {
            let contour = &contours[i];

            let mar = min_area_rect(&contour.points);
            let [tl, tr, dr, _dl] = mar;
            let w = distance(&tl, &tr);
            let h = distance(&tr, &dr);
            let sside = w.min(h);
            if sside < self.min_size + 2.0 {
                continue;
            }
            let score = self.box_score_fast(pred, &mar);
            if score < self.box_threshold {
                continue;
            }
            let area = w * h;
            let arc_length = (w + h) * 2.0;

            let unclip_points = self.unclip(&mar, area, arc_length);
            let bbox = min_area_rect(&unclip_points);

            let res = BoxResult { bbox, score };
            results.push(res);
        }
        Ok(results)
    }

    fn box_score_fast(&self, pred: &Array2<f32>, points: &[Point<i32>; 4]) -> f32 {
        let xs: Vec<i32> = points.iter().map(|p| p.x).collect();
        let ys: Vec<i32> = points.iter().map(|p| p.y).collect();
        let (height, width) = pred.dim();
        let min_x = std::cmp::max(
            0,
            std::cmp::min(xs.iter().min().unwrap().to_owned() as usize, width - 1),
        );
        let max_x = std::cmp::max(
            0,
            std::cmp::min(xs.iter().max().unwrap().to_owned() as usize, width - 1),
        );
        let min_y = std::cmp::max(
            0,
            std::cmp::min(ys.iter().min().unwrap().to_owned() as usize, height - 1),
        );
        let max_y = std::cmp::max(
            0,
            std::cmp::min(ys.iter().max().unwrap().to_owned() as usize, height - 1),
        );
        let mut mask =
            GrayImage::from_pixel((max_x - min_x) as u32, (max_y - min_y) as u32, Luma([0_u8]));
        let all_ps: Vec<Point<i32>> = points
            .iter()
            .map(|p| Point::new(p.x.to_owned() - min_x as i32, p.y.to_owned() - min_y as i32))
            .collect();
        draw_polygon_mut(&mut mask, all_ps.as_slice(), Luma([255_u8]));
        let mut total_score = 0.0;
        let mut n = 0.0;
        for (i, y) in (min_y..max_y).enumerate() {
            for (j, x) in (min_x..max_x).enumerate() {
                if mask.get_pixel(j as u32, i as u32).0[0] == 0_u8 {
                    continue;
                }
                n += 1.0;
                let v = pred.get((y, x)).unwrap();
                total_score += v;
            }
        }
        if n == 0.0 {
            return 0.0;
        }
        total_score / n
    }

    pub fn unclip(&self, points: &[Point<i32>; 4], area: f32, arc: f32) -> Vec<Point<i32>> {
        let distance = area * self.unclip_ratio / arc;
        let offset = ClipperOffset::new(2.0, 0.0, false, false);
        let dpoints: Vec<Point64> = points
            .iter()
            .map(|p| Point64::new(p.x.to_owned() as i64, p.y.to_owned() as i64))
            .collect();
        offset.add_path(
            Path64::new(&dpoints),
            clipper2_sys::JoinType::RoundJoin,
            clipper2_sys::EndType::PolygonEnd,
        );
        let path = offset.execute(distance as f64).get_path(0);

        let mut unclip_points = Vec::new();
        for i in 0..path.len() {
            let p = path.get_point(i);
            unclip_points.push(p);
        }
        let mut res = Vec::new();
        for p in unclip_points {
            let x = p.x.max(0);
            let y = p.y.max(0);
            let np = Point::new(x as i32, y as i32);
            res.push(np);
        }
        res
    }
}

fn distance(p1: &Point<i32>, p2: &Point<i32>) -> f32 {
    let d = ((p1.x as f32 - p2.x as f32) * (p1.x as f32 - p2.x as f32)
        + (p1.y as f32 - p2.y as f32) * (p1.y as f32 - p2.y as f32))
        .sqrt();
    d
}
