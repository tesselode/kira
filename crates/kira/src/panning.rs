use std::ops::{
	Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

use crate::tween::{Tweenable, Value};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Panning(pub f32);

impl Panning {
	pub const LEFT: Self = Self(-1.0);
	pub const CENTER: Self = Self(0.0);
	pub const RIGHT: Self = Self(1.0);
}

impl Default for Panning {
	fn default() -> Self {
		Self::CENTER
	}
}

impl Tweenable for Panning {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		Self(Tweenable::interpolate(a.0, b.0, amount))
	}
}

impl From<f32> for Panning {
	fn from(value: f32) -> Self {
		Self(value)
	}
}

impl From<f32> for Value<Panning> {
	fn from(value: f32) -> Self {
		Self::Fixed(Panning(value))
	}
}

impl From<Panning> for Value<Panning> {
	fn from(value: Panning) -> Self {
		Self::Fixed(value)
	}
}

impl Add<Panning> for Panning {
	type Output = Panning;

	fn add(self, rhs: Panning) -> Self::Output {
		Self(self.0 + rhs.0)
	}
}

impl AddAssign<Panning> for Panning {
	fn add_assign(&mut self, rhs: Panning) {
		self.0 += rhs.0;
	}
}

impl Sub<Panning> for Panning {
	type Output = Panning;

	fn sub(self, rhs: Panning) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}

impl SubAssign<Panning> for Panning {
	fn sub_assign(&mut self, rhs: Panning) {
		self.0 -= rhs.0;
	}
}

impl Mul<f32> for Panning {
	type Output = Panning;

	fn mul(self, rhs: f32) -> Self::Output {
		Self(self.0 * rhs)
	}
}

impl MulAssign<f32> for Panning {
	fn mul_assign(&mut self, rhs: f32) {
		self.0 *= rhs;
	}
}

impl Div<f32> for Panning {
	type Output = Panning;

	fn div(self, rhs: f32) -> Self::Output {
		Self(self.0 / rhs)
	}
}

impl DivAssign<f32> for Panning {
	fn div_assign(&mut self, rhs: f32) {
		self.0 /= rhs;
	}
}

impl Neg for Panning {
	type Output = Panning;

	fn neg(self) -> Self::Output {
		Self(-self.0)
	}
}

impl Rem<f32> for Panning {
	type Output = Panning;

	fn rem(self, rhs: f32) -> Self::Output {
		Self(self.0 % rhs)
	}
}

impl RemAssign<f32> for Panning {
	fn rem_assign(&mut self, rhs: f32) {
		self.0 %= rhs;
	}
}
