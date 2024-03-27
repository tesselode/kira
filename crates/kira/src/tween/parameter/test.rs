use std::time::Duration;

use crate::{
	clock::{
		clock_info::{ClockInfo, MockClockInfoProviderBuilder},
		ClockTime,
	},
	tween::{Tween, Value},
	StartTime,
};

use super::Parameter;

/// Tests that the basic tweening behavior of a `Parameter`
/// works properly.
#[test]
#[allow(clippy::float_cmp)]
fn tweening() {
	let mut parameter = Parameter::new(Value::Fixed(0.0), 0.0);
	let clock_info_provider = MockClockInfoProviderBuilder::new(0).build();

	// value should not be changing yet
	for _ in 0..3 {
		assert_eq!(parameter.value(), 0.0);
		assert!(!parameter.update(1.0, &clock_info_provider));
	}

	parameter.set(
		Value::Fixed(1.0),
		Tween {
			duration: Duration::from_secs(2),
			..Default::default()
		},
	);

	assert!(!parameter.update(1.0, &clock_info_provider));
	assert_eq!(parameter.value(), 0.5);
	assert!(parameter.update(1.0, &clock_info_provider));
	assert_eq!(parameter.value(), 1.0);
	assert!(!parameter.update(1.0, &clock_info_provider));
	assert_eq!(parameter.value(), 1.0);
}

/// Tests that a Parameter with a clock time set as
/// the start time waits for that time before it
/// begins tweening.
#[test]
#[allow(clippy::float_cmp)]
fn waits_for_start_time() {
	let (clock_info_provider, clock_id_1) = {
		let mut builder = MockClockInfoProviderBuilder::new(2);
		let clock_id_1 = builder
			.add(ClockInfo {
				ticking: true,
				ticks: 0,
				fractional_position: 0.0,
			})
			.unwrap();
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 0,
				fractional_position: 0.0,
			})
			.unwrap();
		(builder.build(), clock_id_1)
	};

	let mut parameter = Parameter::new(Value::Fixed(0.0), 0.0);
	parameter.set(
		Value::Fixed(1.0),
		Tween {
			start_time: StartTime::ClockTime(ClockTime {
				clock: clock_id_1,
				ticks: 2,
			}),
			duration: Duration::from_secs(1),
			..Default::default()
		},
	);

	// value should not be changing yet
	for _ in 0..3 {
		assert_eq!(parameter.value(), 0.0);
		assert!(!parameter.update(1.0, &clock_info_provider));
	}

	let clock_info_provider = {
		let mut builder = MockClockInfoProviderBuilder::new(2);
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 1,
				fractional_position: 0.0,
			})
			.unwrap();
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 0,
				fractional_position: 0.0,
			})
			.unwrap();
		builder.build()
	};

	// the tween is set to start at tick 2, so it should not
	// start yet
	for _ in 0..3 {
		assert_eq!(parameter.value(), 0.0);
		assert!(!parameter.update(1.0, &clock_info_provider));
	}

	let clock_info_provider = {
		let mut builder = MockClockInfoProviderBuilder::new(2);
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 1,
				fractional_position: 0.0,
			})
			.unwrap();
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 2,
				fractional_position: 0.0,
			})
			.unwrap();
		builder.build()
	};

	// a different clock reached tick 2, so the tween should not
	// start yet
	for _ in 0..3 {
		assert_eq!(parameter.value(), 0.0);
		assert!(!parameter.update(1.0, &clock_info_provider));
	}

	let clock_info_provider = {
		let mut builder = MockClockInfoProviderBuilder::new(2);
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 2,
				fractional_position: 0.0,
			})
			.unwrap();
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 2,
				fractional_position: 0.0,
			})
			.unwrap();
		builder.build()
	};

	// the tween should start now
	assert!(parameter.update(1.0, &clock_info_provider));
	assert_eq!(parameter.value(), 1.0);
}
