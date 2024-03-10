use std::time::Duration;

use crate::{
	clock::{
		clock_info::{ClockInfo, MockClockInfoProviderBuilder},
		ClockTime,
	},
	modulator::value_provider::MockModulatorValueProviderBuilder,
	resource::Resource,
	tween::{Tween, Value},
	StartTime,
};

use super::{Clock, ClockSpeed};

/// Tests that a `Clock` is stopped when it's first created.
#[test]
fn initially_stopped() {
	let mut clock = create_clock_without_handle(Value::Fixed(ClockSpeed::SecondsPerTick(1.0)));
	let shared = clock.shared();
	for _ in 0..3 {
		assert!(!shared.ticking());
		assert_eq!(shared.ticks(), 0);
		clock.update(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build(),
		);
		clock.on_start_processing();
	}
}

/// Tests that a `Clock` ticks.
#[test]
fn basic_behavior() {
	let mut clock = create_clock_without_handle(Value::Fixed(ClockSpeed::SecondsPerTick(1.0)));
	let shared = clock.shared();
	clock.set_ticking(true);
	for i in 0..3 {
		assert!(shared.ticking());
		assert_eq!(shared.ticks(), i);
		assert_eq!(
			clock.update(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Some(i + 1)
		);
		clock.on_start_processing();
	}
}

/// Tests that a `Clock` can be paused.
#[test]
fn pause() {
	let mut clock = create_clock_without_handle(Value::Fixed(ClockSpeed::SecondsPerTick(1.0)));
	let shared = clock.shared();
	clock.set_ticking(true);
	clock.update(
		1.5,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.ticks(), 1);
	clock.set_ticking(false);
	// the clock should not be ticking
	for _ in 0..3 {
		clock.update(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build(),
		);
		clock.on_start_processing();
		assert!(!shared.ticking());
		assert_eq!(shared.ticks(), 1);
	}
	clock.set_ticking(true);
	// make sure we've preserved the fractional position from before
	// pausing
	clock.update(
		0.4,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.ticks(), 1);
	clock.update(
		0.1,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.ticks(), 2);
}

/// Tests that a `Clock` can be stopped.
#[test]
fn stop() {
	let mut clock = create_clock_without_handle(Value::Fixed(ClockSpeed::SecondsPerTick(1.0)));
	let shared = clock.shared();
	clock.set_ticking(true);
	clock.update(
		1.5,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	clock.set_ticking(false);
	clock.reset();
	// the clock should not be ticking
	for _ in 0..3 {
		clock.update(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build(),
		);
		clock.on_start_processing();
		assert!(!shared.ticking());
		assert_eq!(shared.ticks(), 0);
	}
	clock.set_ticking(true);
	// make sure the fractional position has been reset
	clock.update(
		0.9,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.ticks(), 0);
	clock.update(
		0.1,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.ticks(), 1);
}

/// Tests that the speed of a [`Clock`] can be changed after creation.
#[test]
fn set_speed() {
	let mut clock = create_clock_without_handle(Value::Fixed(ClockSpeed::SecondsPerTick(1.0)));
	let shared = clock.shared();
	clock.set_ticking(true);
	clock.speed.set(
		Value::Fixed(ClockSpeed::SecondsPerTick(0.5)),
		Tween {
			duration: Duration::ZERO,
			..Default::default()
		},
	);
	clock.update(
		1.0,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.ticks(), 2);
	clock.update(
		1.0,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.ticks(), 4);
}

/// Tests that a clock speed tween properly responds to ticks from
/// other clocks when the start time is set to a clock time.
#[test]
fn set_speed_with_clock_time_start() {
	let (clock_info_provider, clock_id) = {
		let mut builder = MockClockInfoProviderBuilder::new(1);
		let clock_id = builder
			.add(ClockInfo {
				ticking: true,
				ticks: 0,
				fractional_position: 0.0,
			})
			.unwrap();
		(builder.build(), clock_id)
	};

	let mut clock = create_clock_without_handle(Value::Fixed(ClockSpeed::SecondsPerTick(1.0)));
	let shared = clock.shared();
	clock.set_ticking(true);
	clock.speed.set(
		Value::Fixed(ClockSpeed::SecondsPerTick(0.5)),
		Tween {
			duration: Duration::ZERO,
			start_time: StartTime::ClockTime(ClockTime {
				clock: clock_id,
				ticks: 1,
			}),
			..Default::default()
		},
	);

	clock.update(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.ticks(), 1);
	clock.update(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.ticks(), 2);

	let clock_info_provider = {
		let mut builder = MockClockInfoProviderBuilder::new(1);
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 1,
				fractional_position: 0.0,
			})
			.unwrap();
		builder.build()
	};

	clock.update(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.ticks(), 4);
	clock.update(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.ticks(), 6);
}

/// Tests that a clock correctly reports its fractional position.
#[test]
fn fractional_position() {
	let mut clock = create_clock_without_handle(Value::Fixed(ClockSpeed::SecondsPerTick(1.0)));
	let shared = clock.shared();
	assert_eq!(shared.fractional_position(), 0.0);
	// the clock is not started yet, so the fractional position should remain at 0
	clock.update(
		1.0,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.fractional_position(), 0.0);
	// start the clock
	clock.set_ticking(true);
	clock.update(
		0.5,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.fractional_position(), 0.5);
	clock.update(
		0.75,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	clock.on_start_processing();
	assert_eq!(shared.fractional_position(), 0.25);
}

fn create_clock_without_handle(speed: Value<ClockSpeed>) -> Clock {
	let (_, command_readers) = super::command_writers_and_readers();
	Clock::new(speed, command_readers)
}
