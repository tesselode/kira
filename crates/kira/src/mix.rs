use std::ops::{
	Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

use crate::{Value, tween::Tweenable};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
/**
An amount to blend the "dry" and "wet" outputs from an effect.

The "dry" signal is the audio before the effect is applied.
The "wet" signal is the audio after the effect is applied.

Valid mix values range from `0.0` to `1.0`, where `0.0` is
the dry signal only, `1.0` is the wet signal only, and `0.5`
is an equal mix of both.
*/
pub struct Mix(pub f32);

impl Mix {
	/// Only output the dry signal.
	pub const DRY: Self = Self(0.0);
	/// Only output the wet signal.
	pub const WET: Self = Self(1.0);
}

impl Tweenable for Mix {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		Self(Tweenable::interpolate(a.0, b.0, amount))
	}
}

impl From<f32> for Mix {
	fn from(value: f32) -> Self {
		Self(value)
	}
}

impl From<f32> for Value<Mix> {
	fn from(value: f32) -> Self {
		Self::Fixed(Mix(value))
	}
}

impl From<Mix> for Value<Mix> {
	fn from(value: Mix) -> Self {
		Self::Fixed(value)
	}
}

impl Add<Mix> for Mix {
	type Output = Mix;

	fn add(self, rhs: Mix) -> Self::Output {
		Self(self.0 + rhs.0)
	}
}

impl AddAssign<Mix> for Mix {
	fn add_assign(&mut self, rhs: Mix) {
		self.0 += rhs.0;
	}
}

impl Sub<Mix> for Mix {
	type Output = Mix;

	fn sub(self, rhs: Mix) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}

impl SubAssign<Mix> for Mix {
	fn sub_assign(&mut self, rhs: Mix) {
		self.0 -= rhs.0;
	}
}

impl Mul<f32> for Mix {
	type Output = Mix;

	fn mul(self, rhs: f32) -> Self::Output {
		Self(self.0 * rhs)
	}
}

impl MulAssign<f32> for Mix {
	fn mul_assign(&mut self, rhs: f32) {
		self.0 *= rhs;
	}
}

impl Div<f32> for Mix {
	type Output = Mix;

	fn div(self, rhs: f32) -> Self::Output {
		Self(self.0 / rhs)
	}
}

impl DivAssign<f32> for Mix {
	fn div_assign(&mut self, rhs: f32) {
		self.0 /= rhs;
	}
}

impl Neg for Mix {
	type Output = Mix;

	fn neg(self) -> Self::Output {
		Self(-self.0)
	}
}

impl Rem<f32> for Mix {
	type Output = Mix;

	fn rem(self, rhs: f32) -> Self::Output {
		Self(self.0 % rhs)
	}
}

impl RemAssign<f32> for Mix {
	fn rem_assign(&mut self, rhs: f32) {
		self.0 %= rhs;
	}
}
