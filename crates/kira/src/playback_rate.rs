use std::ops::{
	Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

use crate::tween::{Tweenable, Value};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PlaybackRate(pub f64);

impl Default for PlaybackRate {
	fn default() -> Self {
		Self(1.0)
	}
}

impl Tweenable for PlaybackRate {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		Self(Tweenable::interpolate(a.0, b.0, amount))
	}
}

impl From<f64> for PlaybackRate {
	fn from(value: f64) -> Self {
		Self(value)
	}
}

impl From<f64> for Value<PlaybackRate> {
	fn from(value: f64) -> Self {
		Self::Fixed(PlaybackRate(value))
	}
}

impl From<PlaybackRate> for Value<PlaybackRate> {
	fn from(value: PlaybackRate) -> Self {
		Self::Fixed(value)
	}
}

impl Add<PlaybackRate> for PlaybackRate {
	type Output = PlaybackRate;

	fn add(self, rhs: PlaybackRate) -> Self::Output {
		Self(self.0 + rhs.0)
	}
}

impl AddAssign<PlaybackRate> for PlaybackRate {
	fn add_assign(&mut self, rhs: PlaybackRate) {
		self.0 += rhs.0;
	}
}

impl Sub<PlaybackRate> for PlaybackRate {
	type Output = PlaybackRate;

	fn sub(self, rhs: PlaybackRate) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}

impl SubAssign<PlaybackRate> for PlaybackRate {
	fn sub_assign(&mut self, rhs: PlaybackRate) {
		self.0 -= rhs.0;
	}
}

impl Mul<f64> for PlaybackRate {
	type Output = PlaybackRate;

	fn mul(self, rhs: f64) -> Self::Output {
		Self(self.0 * rhs)
	}
}

impl MulAssign<f64> for PlaybackRate {
	fn mul_assign(&mut self, rhs: f64) {
		self.0 *= rhs;
	}
}

impl Div<f64> for PlaybackRate {
	type Output = PlaybackRate;

	fn div(self, rhs: f64) -> Self::Output {
		Self(self.0 / rhs)
	}
}

impl DivAssign<f64> for PlaybackRate {
	fn div_assign(&mut self, rhs: f64) {
		self.0 /= rhs;
	}
}

impl Neg for PlaybackRate {
	type Output = PlaybackRate;

	fn neg(self) -> Self::Output {
		Self(-self.0)
	}
}

impl Rem<f64> for PlaybackRate {
	type Output = PlaybackRate;

	fn rem(self, rhs: f64) -> Self::Output {
		Self(self.0 % rhs)
	}
}

impl RemAssign<f64> for PlaybackRate {
	fn rem_assign(&mut self, rhs: f64) {
		self.0 %= rhs;
	}
}
