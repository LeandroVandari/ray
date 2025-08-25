struct Interval {
    min: f32,
    max: f32
}

fn interval() -> Interval {
    return Interval(
        F32_MIN,
        F32_MAX
    );
}

fn interval_size(interval: Interval) -> f32 {
    return interval.max - interval.min;
}

fn interval_contains(interval: Interval, x: f32) -> bool {
    return interval.min <= x && x <= interval.max;
}