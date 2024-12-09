use std::ops::{
	Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

use crate::{
	tween::{Tweenable, Value},
	PlaybackRate,
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
/// A change in pitch in semitones.
///
/// This can be used where [`PlaybackRate`](crate::PlaybackRate)s are expected to control
/// the pitch of a sound.
pub struct Semitones(pub f64);

impl Tweenable for Semitones {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		Self(Tweenable::interpolate(a.0, b.0, amount))
	}
}

impl From<f64> for Semitones {
	fn from(value: f64) -> Self {
		Self(value)
	}
}

impl From<Semitones> for PlaybackRate {
	fn from(value: Semitones) -> Self {
		PlaybackRate(2.0f64.powf(value.0 / 12.0))
	}
}

impl From<Semitones> for Value<PlaybackRate> {
	fn from(value: Semitones) -> Self {
		Self::Fixed(value.into())
	}
}

impl Add<Semitones> for Semitones {
	type Output = Semitones;

	fn add(self, rhs: Semitones) -> Self::Output {
		Self(self.0 + rhs.0)
	}
}

impl AddAssign<Semitones> for Semitones {
	fn add_assign(&mut self, rhs: Semitones) {
		self.0 += rhs.0;
	}
}

impl Sub<Semitones> for Semitones {
	type Output = Semitones;

	fn sub(self, rhs: Semitones) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}

impl SubAssign<Semitones> for Semitones {
	fn sub_assign(&mut self, rhs: Semitones) {
		self.0 -= rhs.0;
	}
}

impl Mul<f64> for Semitones {
	type Output = Semitones;

	fn mul(self, rhs: f64) -> Self::Output {
		Self(self.0 * rhs)
	}
}

impl MulAssign<f64> for Semitones {
	fn mul_assign(&mut self, rhs: f64) {
		self.0 *= rhs;
	}
}

impl Div<f64> for Semitones {
	type Output = Semitones;

	fn div(self, rhs: f64) -> Self::Output {
		Self(self.0 / rhs)
	}
}

impl DivAssign<f64> for Semitones {
	fn div_assign(&mut self, rhs: f64) {
		self.0 /= rhs;
	}
}

impl Neg for Semitones {
	type Output = Semitones;

	fn neg(self) -> Self::Output {
		Self(-self.0)
	}
}

impl Rem<f64> for Semitones {
	type Output = Semitones;

	fn rem(self, rhs: f64) -> Self::Output {
		Self(self.0 % rhs)
	}
}

impl RemAssign<f64> for Semitones {
	fn rem_assign(&mut self, rhs: f64) {
		self.0 %= rhs;
	}
}

#[cfg(test)]
#[test]
#[allow(clippy::float_cmp)]
fn test() {
	/// A table of semitone differences to pitch factors.
	/// Values calculated from http://www.sengpielaudio.com/calculator-centsratio.htm
	const TEST_CALCULATIONS: [(Semitones, PlaybackRate); 5] = [
		(Semitones(0.0), PlaybackRate(1.0)),
		(Semitones(1.0), PlaybackRate(1.059463)),
		(Semitones(2.0), PlaybackRate(1.122462)),
		(Semitones(-1.0), PlaybackRate(0.943874)),
		(Semitones(-2.0), PlaybackRate(0.890899)),
	];

	for (semitones, playback_rate) in TEST_CALCULATIONS {
		assert!((PlaybackRate::from(semitones).0 - playback_rate.0).abs() < 0.00001);
	}
}
