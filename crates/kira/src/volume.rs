mod amplitude;
mod decibels;

pub use amplitude::*;
pub use decibels::*;

#[cfg(test)]
#[test]
#[allow(clippy::float_cmp)]
fn test() {
	/// A table of dB values to the corresponding amplitudes.
	// Data gathered from https://www.silisoftware.com/tools/db.php
	const TEST_CALCULATIONS: [(Decibels, Amplitude); 6] = [
		(Decibels(0.0), Amplitude(1.0)),
		(Decibels(3.0), Amplitude(1.4125375446227544)),
		(Decibels(12.0), Amplitude(3.9810717055349722)),
		(Decibels(-3.0), Amplitude(0.7079457843841379)),
		(Decibels(-12.0), Amplitude(0.251188643150958)),
		(Decibels::MIN, Amplitude(0.0)),
	];

	for (db, amplitude) in TEST_CALCULATIONS {
		assert!((Decibels::from(amplitude) - db).0.abs() < 0.00001);
		assert!((Amplitude::from(db) - amplitude).0.abs() < 0.00001);
	}

	// test some special cases
	assert_eq!(
		Amplitude::from(Decibels::MIN - Decibels(100.0)),
		Amplitude(0.0)
	);
	assert_eq!(Decibels::from(Amplitude(-1.0)), Decibels::MIN);
}
