use std::ops::RangeInclusive;

pub fn scale_float(
    source: f32,
    source_range: RangeInclusive<f32>,
    target_range: RangeInclusive<f32>,
) -> f32 {
    (source - source_range.start()) / (source_range.end() - source_range.start())
        * (target_range.end() - target_range.start())
        + target_range.start()
}
