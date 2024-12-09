use std::f64::consts::FRAC_1_SQRT_2;

use approx::assert_relative_eq;

use crate::modulator::lfo::Waveform;

#[test]
fn sine() {
	test_waveform(
		Waveform::Sine,
		[
			0.0,
			FRAC_1_SQRT_2,
			1.0,
			FRAC_1_SQRT_2,
			0.0,
			-FRAC_1_SQRT_2,
			-1.0,
			-FRAC_1_SQRT_2,
		],
	);
}

#[test]
fn triangle() {
	test_waveform(
		Waveform::Triangle,
		[0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5],
	);
}

#[test]
fn saw() {
	test_waveform(
		Waveform::Saw,
		[0.0, 0.25, 0.5, 0.75, -1.0, -0.75, -0.5, -0.25],
	);
}

#[test]
fn pulse() {
	test_waveform(
		Waveform::Pulse { width: 0.25 },
		[1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0],
	);
	test_waveform(
		Waveform::Pulse { width: 0.75 },
		[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0],
	);
}

fn test_waveform(waveform: Waveform, values: [f64; 8]) {
	for (i, value) in values.iter().enumerate() {
		assert_relative_eq!(waveform.value(i as f64 / 8.0), *value);
	}
}
