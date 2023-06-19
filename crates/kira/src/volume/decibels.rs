use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::tween::{ModulatorMapping, Tweenable, Value};

use super::Amplitude;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A change in volume of a sound in dBFS.
pub struct Decibels(pub f64);

impl Decibels {
	/// The minimum decibel value at which a sound is considered
	/// silent.
	pub const MIN: Self = Self(-60.0);
}

impl Add for Decibels {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self(self.0 + rhs.0)
	}
}

impl AddAssign for Decibels {
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0;
	}
}

impl Sub for Decibels {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}

impl SubAssign for Decibels {
	fn sub_assign(&mut self, rhs: Self) {
		self.0 -= rhs.0;
	}
}

impl Mul<f64> for Decibels {
	type Output = Self;

	fn mul(self, rhs: f64) -> Self::Output {
		Self(self.0 * rhs)
	}
}

impl MulAssign<f64> for Decibels {
	fn mul_assign(&mut self, rhs: f64) {
		self.0 *= rhs;
	}
}

impl Div<f64> for Decibels {
	type Output = Self;

	fn div(self, rhs: f64) -> Self::Output {
		Self(self.0 / rhs)
	}
}

impl DivAssign<f64> for Decibels {
	fn div_assign(&mut self, rhs: f64) {
		self.0 /= rhs;
	}
}

impl Neg for Decibels {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self(-self.0)
	}
}

impl From<Amplitude> for Decibels {
	fn from(amplitude: Amplitude) -> Self {
		if amplitude <= Amplitude(0.0) {
			return Self::MIN;
		}
		Decibels(20.0 * amplitude.0.log10())
	}
}

impl From<f64> for Decibels {
	fn from(amplitude: f64) -> Self {
		Amplitude(amplitude).into()
	}
}

impl From<Decibels> for Value<Decibels> {
	fn from(decibels: Decibels) -> Self {
		Self::Fixed(decibels)
	}
}

impl From<Amplitude> for Value<Decibels> {
	fn from(amplitude: Amplitude) -> Self {
		Self::Fixed(amplitude.into())
	}
}

impl From<f64> for Value<Decibels> {
	fn from(amplitude: f64) -> Self {
		Self::Fixed(amplitude.into())
	}
}

impl Tweenable for Decibels {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		a + (b - a) * amount
	}
}

impl Default for ModulatorMapping<Decibels> {
	fn default() -> Self {
		Self {
			input_range: (0.0, 1.0),
			output_range: (Decibels::MIN, Decibels(0.0)),
			clamp_bottom: false,
			clamp_top: false,
		}
	}
}
