use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::tween::{Tweenable, Value};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
/// Represents a change in volume.
///
/// Higher values increase the volume and lower values decrease it.
/// Setting the volume of a sound to -60dB or lower makes it silent.
pub struct Decibels(pub f32);

impl Decibels {
	/// The minimum decibel value at which a sound is considered
	/// silent.
	pub const SILENCE: Self = Self(-60.0);
	/// The decibel value that produces no change in volume.
	pub const IDENTITY: Self = Self(0.0);

	/// Converts decibels to amplitude, a linear volume measurement.
	///
	/// This returns a number from `0.0`-`1.0` that you can multiply
	/// a singal by to change its volume.
	pub fn as_amplitude(self) -> f32 {
		// adding a special case for db == 0.0 improves
		// performance in the sound playback benchmarks
		// by about 7%
		if self == Self(0.0) {
			return 1.0;
		}
		if self <= Self::SILENCE {
			return 0.0;
		}
		10.0f32.powf(self.0 / 20.0)
	}
}

impl Default for Decibels {
	fn default() -> Self {
		Self::IDENTITY
	}
}

impl Tweenable for Decibels {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		Self(Tweenable::interpolate(a.0, b.0, amount))
	}
}

impl From<f32> for Decibels {
	fn from(value: f32) -> Self {
		Self(value)
	}
}

impl From<f32> for Value<Decibels> {
	fn from(value: f32) -> Self {
		Value::Fixed(Decibels(value))
	}
}

impl From<Decibels> for Value<Decibels> {
	fn from(value: Decibels) -> Self {
		Value::Fixed(value)
	}
}

impl Add<Decibels> for Decibels {
	type Output = Decibels;

	fn add(self, rhs: Decibels) -> Self::Output {
		Self(self.0 + rhs.0)
	}
}

impl AddAssign<Decibels> for Decibels {
	fn add_assign(&mut self, rhs: Decibels) {
		self.0 += rhs.0;
	}
}

impl Sub<Decibels> for Decibels {
	type Output = Decibels;

	fn sub(self, rhs: Decibels) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}

impl SubAssign<Decibels> for Decibels {
	fn sub_assign(&mut self, rhs: Decibels) {
		self.0 -= rhs.0;
	}
}

#[cfg(test)]
#[test]
#[allow(clippy::float_cmp)]
fn test() {
	/// A table of dB values to the corresponding amplitudes.
	// Data gathered from https://www.silisoftware.com/tools/db.php
	const TEST_CALCULATIONS: [(Decibels, f32); 6] = [
		(Decibels::IDENTITY, 1.0),
		(Decibels(3.0), 1.4125376),
		(Decibels(12.0), 3.9810717),
		(Decibels(-3.0), 0.70794576),
		(Decibels(-12.0), 0.25118864),
		(Decibels::SILENCE, 0.0),
	];

	for (dbfs, amplitude) in TEST_CALCULATIONS {
		assert!((dbfs.as_amplitude() - amplitude).abs() < 0.00001);
	}

	// test some special cases
	assert_eq!((Decibels::SILENCE - Decibels(100.0)).as_amplitude(), 0.0);
}
