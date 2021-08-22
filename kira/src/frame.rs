use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A stereo audio sample.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Frame {
	/// The sample for the left channel.
	pub left: f32,
	/// The sample for the right channel.
	pub right: f32,
}

impl Frame {
	/// A [`Frame`] with both the left and right samples
	/// set to `0.0`.
	pub const ZERO: Frame = Frame {
		left: 0.0,
		right: 0.0,
	};

	/// Creates a frame with the given left and right values.
	pub fn new(left: f32, right: f32) -> Self {
		Self { left, right }
	}

	/// Creates a frame with both the left and right channels set
	/// to the same value.
	pub fn from_mono(value: f32) -> Self {
		Self::new(value, value)
	}

	/// Creates a frame from `i32`s with the given bit depth.
	pub fn from_i32(left: i32, right: i32, bit_depth: u32) -> Self {
		let max_int = (1 << bit_depth) / 2;
		let scale = 1.0 / max_int as f32;
		Self::new(left as f32 * scale, right as f32 * scale)
	}

	/// Pans a frame to the left or right.
	///
	/// An `x` of 0 represents a hard left panning, an `x` of 1
	/// represents a hard right panning.
	pub fn panned(self, x: f32) -> Self {
		Self::new(self.left * (1.0 - x).sqrt(), self.right * x.sqrt())
	}
}

impl Add for Frame {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self::new(self.left + rhs.left, self.right + rhs.right)
	}
}

impl AddAssign for Frame {
	fn add_assign(&mut self, rhs: Self) {
		self.left += rhs.left;
		self.right += rhs.right;
	}
}

impl Sub for Frame {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self::new(self.left - rhs.left, self.right - rhs.right)
	}
}

impl SubAssign for Frame {
	fn sub_assign(&mut self, rhs: Self) {
		self.left -= rhs.left;
		self.right -= rhs.right;
	}
}

impl Mul<f32> for Frame {
	type Output = Self;

	fn mul(self, rhs: f32) -> Self::Output {
		Self::new(self.left * rhs, self.right * rhs)
	}
}

impl MulAssign<f32> for Frame {
	fn mul_assign(&mut self, rhs: f32) {
		self.left *= rhs;
		self.right *= rhs;
	}
}

impl Div<f32> for Frame {
	type Output = Self;

	fn div(self, rhs: f32) -> Self::Output {
		Self::new(self.left / rhs, self.right / rhs)
	}
}

impl DivAssign<f32> for Frame {
	fn div_assign(&mut self, rhs: f32) {
		self.left /= rhs;
		self.right /= rhs;
	}
}

impl Neg for Frame {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self::new(-self.left, -self.right)
	}
}
