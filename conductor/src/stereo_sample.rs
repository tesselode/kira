use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Debug, Copy, Clone)]
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

	pub fn from_i32(left: i32, right: i32, bit_depth: u32) -> Self {
		let max_int = (1 << bit_depth) / 2;
		let scale = 1.0 / max_int as f32;
		Self::new(left as f32 * scale, right as f32 * scale)
	}

	pub fn atan(self) -> Self {
		Self {
			left: self.left.atan(),
			right: self.right.atan(),
		}
	}
}

impl Add for StereoSample {
	type Output = Self;

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

impl Sub for StereoSample {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self::new(self.left - rhs.left, self.right - rhs.right)
	}
}

impl SubAssign for StereoSample {
	fn sub_assign(&mut self, rhs: Self) {
		self.left -= rhs.left;
		self.right -= rhs.right;
	}
}

impl Mul<f32> for StereoSample {
	type Output = Self;

	fn mul(self, rhs: f32) -> Self::Output {
		Self::new(self.left * rhs, self.right * rhs)
	}
}

impl MulAssign<f32> for StereoSample {
	fn mul_assign(&mut self, rhs: f32) {
		self.left *= rhs;
		self.right *= rhs;
	}
}

impl Div<f32> for StereoSample {
	type Output = Self;

	fn div(self, rhs: f32) -> Self::Output {
		Self::new(self.left / rhs, self.right / rhs)
	}
}

impl DivAssign<f32> for StereoSample {
	fn div_assign(&mut self, rhs: f32) {
		self.left /= rhs;
		self.right /= rhs;
	}
}

impl Neg for StereoSample {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self::new(-self.left, -self.right)
	}
}
