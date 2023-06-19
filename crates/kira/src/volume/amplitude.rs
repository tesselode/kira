use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::tween::Tweenable;

use super::Decibels;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A change in volume of a sound as a factor of the original volume.
pub struct Amplitude(pub f64);

impl Add for Amplitude {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self(self.0 + rhs.0)
	}
}

impl AddAssign for Amplitude {
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0;
	}
}

impl Sub for Amplitude {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}

impl SubAssign for Amplitude {
	fn sub_assign(&mut self, rhs: Self) {
		self.0 -= rhs.0;
	}
}

impl Mul<f64> for Amplitude {
	type Output = Self;

	fn mul(self, rhs: f64) -> Self::Output {
		Self(self.0 * rhs)
	}
}

impl MulAssign<f64> for Amplitude {
	fn mul_assign(&mut self, rhs: f64) {
		self.0 *= rhs;
	}
}

impl Div<f64> for Amplitude {
	type Output = Self;

	fn div(self, rhs: f64) -> Self::Output {
		Self(self.0 / rhs)
	}
}

impl DivAssign<f64> for Amplitude {
	fn div_assign(&mut self, rhs: f64) {
		self.0 /= rhs;
	}
}

impl Neg for Amplitude {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self(-self.0)
	}
}

impl From<Decibels> for Amplitude {
	fn from(decibels: Decibels) -> Self {
		// adding a special case for db == 0.0 improves
		// performance in the sound playback benchmarks
		// by about 7%
		if decibels == Decibels(0.0) {
			return Self(1.0);
		}
		if decibels <= Decibels::MIN {
			return Self(0.0);
		}
		Self(10.0f64.powf(decibels.0 / 20.0))
	}
}

impl Tweenable for Amplitude {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		a + (b - a) * amount
	}
}
