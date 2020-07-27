#[derive(Debug)]
pub struct StereoSample {
	pub left: f32,
	pub right: f32,
}

impl StereoSample {
	pub fn new(left: f32, right: f32) -> Self {
		Self { left, right }
	}

	pub fn from_mono(value: f32) -> Self {
		Self::new(value, value)
	}
}
