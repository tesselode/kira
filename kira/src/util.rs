pub fn lerp(a: f64, b: f64, amount: f64) -> f64 {
	a + (b - a) * amount
}

pub fn inverse_lerp(start: f64, end: f64, point: f64) -> f64 {
	(point - start) / (end - start)
}
