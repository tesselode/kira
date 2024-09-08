use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::tween::{Mapping, Tweenable, Value};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dbfs(pub f32);

impl Dbfs {
	/// The minimum decibel value at which a sound is considered
	/// silent.
	pub const MIN: Self = Self(-60.0);
	pub const MAX: Self = Self(0.0);

	/// Returns the volume as an amplitude.
	pub fn as_amplitude(self) -> f32 {
		// adding a special case for db == 0.0 improves
		// performance in the sound playback benchmarks
		// by about 7%
		if self == Self(0.0) {
			return 1.0;
		}
		if self <= Self::MIN {
			return 0.0;
		}
		10.0f32.powf(self.0 / 20.0)
	}
}

impl Default for Dbfs {
	fn default() -> Self {
		Self::MAX
	}
}

impl Tweenable for Dbfs {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		Self(Tweenable::interpolate(a.0, b.0, amount))
	}
}

impl From<f32> for Dbfs {
	fn from(value: f32) -> Self {
		Self(value)
	}
}

impl From<f32> for Value<Dbfs> {
	fn from(value: f32) -> Self {
		Value::Fixed(Dbfs(value))
	}
}

impl From<Dbfs> for Value<Dbfs> {
	fn from(value: Dbfs) -> Self {
		Value::Fixed(value)
	}
}

impl Add<Dbfs> for Dbfs {
	type Output = Dbfs;

	fn add(self, rhs: Dbfs) -> Self::Output {
		Self(self.0 + rhs.0)
	}
}

impl AddAssign<Dbfs> for Dbfs {
	fn add_assign(&mut self, rhs: Dbfs) {
		self.0 += rhs.0;
	}
}

impl Sub<Dbfs> for Dbfs {
	type Output = Dbfs;

	fn sub(self, rhs: Dbfs) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}

impl SubAssign<Dbfs> for Dbfs {
	fn sub_assign(&mut self, rhs: Dbfs) {
		self.0 -= rhs.0;
	}
}

impl Default for Mapping<Dbfs> {
	fn default() -> Self {
		Self {
			input_range: (0.0, 1.0),
			output_range: (Dbfs::MIN, Dbfs::MAX),
			clamp_bottom: true,
			clamp_top: true,
		}
	}
}

#[cfg(test)]
#[test]
#[allow(clippy::float_cmp)]
fn test() {
	/// A table of dB values to the corresponding amplitudes.
	// Data gathered from https://www.silisoftware.com/tools/db.php
	const TEST_CALCULATIONS: [(Dbfs, f32); 6] = [
		(Dbfs::MAX, 1.0),
		(Dbfs(3.0), 1.4125375446227544),
		(Dbfs(12.0), 3.9810717055349722),
		(Dbfs(-3.0), 0.7079457843841379),
		(Dbfs(-12.0), 0.251188643150958),
		(Dbfs::MIN, 0.0),
	];

	for (dbfs, amplitude) in TEST_CALCULATIONS {
		assert!((dbfs.as_amplitude() - amplitude).abs() < 0.00001);
	}

	// test some special cases
	assert_eq!((Dbfs::MIN - Dbfs(100.0)).as_amplitude(), 0.0);
}
