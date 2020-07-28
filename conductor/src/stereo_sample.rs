use std::ops::{Add, AddAssign};

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

impl Add for StereoSample {
	type Output = StereoSample;

	fn add(self, rhs: Self) -> Self::Output {
		Self::new(self.left + rhs.left, self.right + rhs.right)
	}
}

impl AddAssign for StereoSample {
	fn add_assign(&mut self, rhs: Self) {
		self.left += rhs.left;
		self.right += rhs.right;
	}
}
