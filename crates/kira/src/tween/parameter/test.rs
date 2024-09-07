use std::time::Duration;

use crate::{
	clock::ClockTime,
	info::MockInfoBuilder,
	tween::{Tween, Value},
	StartTime,
};

use super::{Mapping, Parameter};

/// Tests that the basic tweening behavior of a `Parameter`
/// works properly.
#[test]
#[allow(clippy::float_cmp)]
fn tweening() {
	let mut parameter = Parameter::new(Value::Fixed(0.0), 0.0);
	let info = MockInfoBuilder::new(None).build();

	// value should not be changing yet
	for _ in 0..3 {
		assert_eq!(parameter.value(), 0.0);
		assert!(!parameter.update(1.0, &info));
	}

	parameter.set(
		Value::Fixed(1.0),
		Tween {
			duration: Duration::from_secs(2),
			..Default::default()
		},
	);

	assert!(!parameter.update(1.0, &info));
	assert_eq!(parameter.value(), 0.5);
	assert!(parameter.update(1.0, &info));
	assert_eq!(parameter.value(), 1.0);
	assert!(!parameter.update(1.0, &info));
	assert_eq!(parameter.value(), 1.0);
}

/// Tests that a `Parameter` with a delayed start time waits for
/// that time before it begins tweening.
#[test]
#[allow(clippy::float_cmp)]
fn waits_for_delay() {
	let info = MockInfoBuilder::new(None).build();

	let mut parameter = Parameter::new(Value::Fixed(0.0), 0.0);
	parameter.set(
		Value::Fixed(1.0),
		Tween {
			start_time: StartTime::Delayed(Duration::from_secs(2)),
			duration: Duration::from_secs(1),
			..Default::default()
		},
	);

	// value should not be changing yet
	for _ in 0..2 {
		assert_eq!(parameter.value(), 0.0);
		assert!(!parameter.update(1.0, &info));
	}

	// the tween should start now
	assert!(parameter.update(1.0, &info));
	assert_eq!(parameter.value(), 1.0);
}

/// Tests that a `Parameter` with a clock time set as
/// the start time waits for that time before it
/// begins tweening.
#[test]
#[allow(clippy::float_cmp)]
fn waits_for_start_time() {
	let mut info_builder = MockInfoBuilder::new(None);
	let clock_id_1 = info_builder.add_clock(true, 0, 0.0);
	let info = info_builder.build();

	let mut parameter = Parameter::new(Value::Fixed(0.0), 0.0);
	parameter.set(
		Value::Fixed(1.0),
		Tween {
			start_time: StartTime::ClockTime(ClockTime {
				clock: clock_id_1,
				ticks: 2,
				fraction: 0.0,
			}),
			duration: Duration::from_secs(1),
			..Default::default()
		},
	);

	// value should not be changing yet
	for _ in 0..3 {
		assert_eq!(parameter.value(), 0.0);
		assert!(!parameter.update(1.0, &info));
	}

	let info = {
		let mut builder = MockInfoBuilder::new(None);
		builder.add_clock(true, 1, 0.0);
		builder.add_clock(true, 0, 0.0);
		builder.build()
	};

	// the tween is set to start at tick 2, so it should not
	// start yet
	for _ in 0..3 {
		assert_eq!(parameter.value(), 0.0);
		assert!(!parameter.update(1.0, &info));
	}

	let info = {
		let mut builder = MockInfoBuilder::new(None);
		builder.add_clock(true, 1, 0.0);
		builder.add_clock(true, 2, 0.0);
		builder.build()
	};

	// a different clock reached tick 2, so the tween should not
	// start yet
	for _ in 0..3 {
		assert_eq!(parameter.value(), 0.0);
		assert!(!parameter.update(1.0, &info));
	}

	let info = {
		let mut builder = MockInfoBuilder::new(None);
		builder.add_clock(true, 2, 0.0);
		builder.add_clock(true, 2, 0.0);
		builder.build()
	};

	// the tween should start now
	assert!(parameter.update(1.0, &info));
	assert_eq!(parameter.value(), 1.0);
}

/// Tests that a parameter can smoothly tween from a fixed value
/// to the value of a modulator (while the modulator value is changing).
#[test]
#[allow(clippy::float_cmp)]
fn tweens_to_modulator_values() {
	let mut info_builder = MockInfoBuilder::new(None);
	let modulator_id = info_builder.add_modulator(0.0);
	let mut parameter = Parameter::new(Value::Fixed(0.0), 0.0);
	parameter.set(
		Value::FromModulator {
			id: modulator_id,
			mapping: Mapping {
				input_range: (0.0, 1.0),
				output_range: (0.0, 1.0),
				clamp_bottom: false,
				clamp_top: false,
			},
		},
		Tween {
			duration: Duration::from_secs(1),
			..Default::default()
		},
	);

	for i in 1..=4 {
		let time = i as f64 / 4.0;
		let info = {
			let mut builder = MockInfoBuilder::new(None);
			builder.add_modulator(time);
			builder.build()
		};
		parameter.update(0.25, &info);
		assert_eq!(parameter.value(), time * time);
	}
}
