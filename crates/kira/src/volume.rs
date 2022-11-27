use crate::tween::Tweenable;

/// A change in volume of a sound.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Volume {
	/// All samples are multiplied by the specified factor.
	Amplitude(f64),
	/// The volume is adjusted by the given number of decibels.
	Decibels(f64),
}

impl Volume {
	/// The minimum decibel value at which a sound is considered
	/// silent.
	pub const MIN_DECIBELS: f64 = -60.0;

	/// Returns the volume as an amplitude.
	pub fn as_amplitude(&self) -> f64 {
		match self {
			Volume::Amplitude(amplitude) => *amplitude,
			Volume::Decibels(db) => {
				// adding a special case for db == 0.0 improves
				// performance in the sound playback benchmarks
				// by about 7%
				if *db == 0.0 {
					return 1.0;
				}
				if *db <= Self::MIN_DECIBELS {
					return 0.0;
				}
				10.0f64.powf(*db / 20.0)
			}
		}
	}

	/// Returns the volume as a difference in the number of decibels.
	pub fn as_decibels(&self) -> f64 {
		match self {
			Volume::Amplitude(amplitude) => {
				if *amplitude <= 0.0 {
					return Self::MIN_DECIBELS;
				}
				20.0 * amplitude.log10()
			}
			Volume::Decibels(db) => *db,
		}
	}
}

impl From<f64> for Volume {
	fn from(amplitude: f64) -> Self {
		Self::Amplitude(amplitude)
	}
}

impl Tweenable for Volume {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		match b {
			Volume::Amplitude(b) => {
				Volume::Amplitude(Tweenable::interpolate(a.as_amplitude(), b, amount))
			}
			Volume::Decibels(b) => {
				Volume::Decibels(Tweenable::interpolate(a.as_decibels(), b, amount))
			}
		}
	}
}

#[cfg(test)]
#[test]
#[allow(clippy::float_cmp)]
fn test() {
	/// A table of dB values to the corresponding amplitudes.
	// Data gathered from https://www.silisoftware.com/tools/db.php
	const TEST_CALCULATIONS: [(f64, f64); 6] = [
		(0.0, 1.0),
		(3.0, 1.4125375446227544),
		(12.0, 3.9810717055349722),
		(-3.0, 0.7079457843841379),
		(-12.0, 0.251188643150958),
		(Volume::MIN_DECIBELS, 0.0),
	];

	for (db, amplitude) in TEST_CALCULATIONS {
		assert_eq!(Volume::Amplitude(amplitude).as_amplitude(), amplitude);
		assert!((Volume::Amplitude(amplitude).as_decibels() - db).abs() < 0.00001);
		assert_eq!(Volume::Decibels(db).as_decibels(), db);
		assert!((Volume::Decibels(db).as_amplitude() - amplitude).abs() < 0.00001);
	}

	// test some special cases
	assert_eq!(
		Volume::Decibels(Volume::MIN_DECIBELS - 100.0).as_amplitude(),
		0.0
	);
	assert_eq!(Volume::Amplitude(-1.0).as_decibels(), Volume::MIN_DECIBELS);
}
