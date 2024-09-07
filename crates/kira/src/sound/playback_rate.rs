use crate::tween::{Mapping, Tweenable, Value};

/// How quickly a sound is played.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlaybackRate {
	/// The sound is played at a speed factor of the
	/// original playback rate.
	///
	/// For example, `PlaybackRate::Factor(2.0)` means
	/// the sound is played twice as fast as normal.
	Factor(f64),
	/// The sound is played faster or slower so that the
	/// pitch of the sound is adjusted by the given number
	/// of semitones.
	Semitones(f64),
}

impl PlaybackRate {
	/// Returns the playback rate as a factor of the original
	/// playback rate.
	#[must_use]
	pub fn as_factor(&self) -> f64 {
		match self {
			PlaybackRate::Factor(factor) => *factor,
			PlaybackRate::Semitones(semitones) => 2.0f64.powf(*semitones / 12.0),
		}
	}

	/// Returns the number of semitones of pitch difference this
	/// playback rate will cause.
	#[must_use]
	pub fn as_semitones(&self) -> f64 {
		match self {
			PlaybackRate::Factor(factor) => 12.0 * factor.log2(),
			PlaybackRate::Semitones(semitones) => *semitones,
		}
	}
}

impl Default for PlaybackRate {
	fn default() -> Self {
		Self::Factor(1.0)
	}
}

impl From<f64> for PlaybackRate {
	fn from(factor: f64) -> Self {
		Self::Factor(factor)
	}
}

impl From<PlaybackRate> for Value<PlaybackRate> {
	fn from(playback_rate: PlaybackRate) -> Self {
		Self::Fixed(playback_rate)
	}
}

impl From<f64> for Value<PlaybackRate> {
	fn from(factor: f64) -> Self {
		Self::Fixed(PlaybackRate::Factor(factor))
	}
}

impl Tweenable for PlaybackRate {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		match b {
			PlaybackRate::Factor(b) => {
				PlaybackRate::Factor(Tweenable::interpolate(a.as_factor(), b, amount))
			}
			PlaybackRate::Semitones(b) => {
				PlaybackRate::Semitones(Tweenable::interpolate(a.as_semitones(), b, amount))
			}
		}
	}
}

impl Default for Mapping<PlaybackRate> {
	fn default() -> Self {
		Self {
			input_range: (0.0, 1.0),
			output_range: (PlaybackRate::Factor(0.0), PlaybackRate::Factor(1.0)),
			clamp_bottom: false,
			clamp_top: false,
		}
	}
}

#[cfg(test)]
#[test]
#[allow(clippy::float_cmp)]
fn test() {
	/// A table of semitone differences to pitch factors.
	/// Values calculated from http://www.sengpielaudio.com/calculator-centsratio.htm
	const TEST_CALCULATIONS: [(f64, f64); 5] = [
		(0.0, 1.0),
		(1.0, 1.059463),
		(2.0, 1.122462),
		(-1.0, 0.943874),
		(-2.0, 0.890899),
	];

	for (semitones, factor) in TEST_CALCULATIONS {
		assert_eq!(PlaybackRate::Factor(factor).as_factor(), factor);
		assert!((PlaybackRate::Factor(factor).as_semitones() - semitones).abs() < 0.00001);

		assert_eq!(PlaybackRate::Semitones(semitones).as_semitones(), semitones);
		assert!((PlaybackRate::Semitones(semitones).as_factor() - factor).abs() < 0.00001);
	}
}
