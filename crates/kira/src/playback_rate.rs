use crate::tween::Tweenable;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackRate {
	Factor(f64),
	Semitones(f64),
}

impl PlaybackRate {
	pub fn as_factor(&self) -> f64 {
		match self {
			PlaybackRate::Factor(factor) => *factor,
			PlaybackRate::Semitones(semitones) => 2.0f64.powf(*semitones / 12.0),
		}
	}

	pub fn as_semitones(&self) -> f64 {
		match self {
			PlaybackRate::Factor(factor) => 12.0 * factor.log2(),
			PlaybackRate::Semitones(semitones) => *semitones,
		}
	}
}

impl From<f64> for PlaybackRate {
	fn from(factor: f64) -> Self {
		Self::Factor(factor)
	}
}

impl Tweenable for PlaybackRate {
	fn lerp(a: Self, b: Self, amount: f64) -> Self {
		match b {
			PlaybackRate::Factor(b) => {
				PlaybackRate::Factor(Tweenable::lerp(a.as_factor(), b, amount))
			}
			PlaybackRate::Semitones(b) => {
				PlaybackRate::Semitones(Tweenable::lerp(a.as_semitones(), b, amount))
			}
		}
	}
}
