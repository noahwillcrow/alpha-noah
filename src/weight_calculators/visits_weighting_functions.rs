#[allow(dead_code)]
pub fn zero(_: i32, _: i32, _: i32) -> f32 {
    return 0.0;
}

#[allow(dead_code)]
pub fn one(_: i32, _: i32, _: i32) -> f32 {
    return 1.0;
}

pub fn difference_from_max(visits_count: i32, _: i32, max_available_visits: i32) -> f32 {
    return (max_available_visits - visits_count) as f32;
}
