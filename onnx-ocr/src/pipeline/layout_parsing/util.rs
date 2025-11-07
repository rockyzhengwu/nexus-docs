pub fn caculate_overlap_ratio(
    rect1: &[f32; 4],
    rect1_area: f32,
    rect2: &[f32; 4],
    rect2_area: f32,
) -> f32 {
    let [r1_x1, r1_y1, r1_x2, r1_y2] = rect1;
    let [r2_x1, r2_y1, r2_x2, r2_y2] = rect2;
    let inter_x1 = r1_x1.max(*r2_x1);
    let inter_y1 = r1_y1.max(*r2_y1);
    let inter_x2 = r1_x2.min(*r2_x2);
    let inter_y2 = r1_y2.min(*r2_y2);
    let inter_width = (inter_x2 - inter_x1).max(0.0);
    let inter_height = (inter_y2 - inter_y1).max(0.0);
    let inter_area = inter_width * inter_height;
    if inter_y2 < inter_y1 || inter_x2 < inter_x1 {
        return 0.0;
    }

    let min_area = rect1_area.min(rect2_area);
    if min_area == 0.0 {
        return 0.0;
    }
    inter_area / min_area
}
