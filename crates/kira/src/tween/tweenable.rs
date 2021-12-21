pub trait Tweenable: Copy {
	fn lerp(a: Self, b: Self, amount: f64) -> Self;
}

impl Tweenable for f64 {
	fn lerp(a: Self, b: Self, amount: f64) -> Self {
		a + (b - a) * amount
	}
}
