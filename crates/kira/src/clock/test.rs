use std::time::Duration;

use crate::{
	arena::Arena,
	clock::{clock_info::MockClockInfoProviderBuilder, ClockTime},
	listener::MockListenerInfoProviderBuilder,
	modulator::value_provider::MockModulatorValueProviderBuilder,
	tween::{Tween, Value},
	StartTime,
};

use super::{Clock, ClockId, ClockSpeed};

/// Tests that a `Clock` is stopped when it's first created.
#[test]
fn initially_stopped() {
	let (mut clock, handle) = Clock::new(
		Value::Fixed(ClockSpeed::SecondsPerTick(1.0)),
		fake_clock_id(),
	);
	for _ in 0..3 {
		assert!(!handle.ticking());
		assert_eq!(handle.time().ticks, 0);
		clock.update(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build(),
			&MockListenerInfoProviderBuilder::new(None, 0).build(),
		);
		clock.on_start_processing();
	}
}

/// Tests that a `Clock` ticks.
#[test]
fn basic_behavior() {
	let (mut clock, mut handle) = Clock::new(
		Value::Fixed(ClockSpeed::SecondsPerTick(1.0)),
		fake_clock_id(),
	);
	handle.start();
	clock.on_start_processing();
	for i in 0..3 {
		assert!(handle.ticking());
		assert_eq!(handle.time().ticks, i);
		assert_eq!(
			clock.update(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build(),
				&MockListenerInfoProviderBuilder::new(None, 0).build()
			),
			Some(i + 1)
		);
		clock.on_start_processing();
	}
}

/// Tests that a `Clock` can be paused.
#[test]
fn pause() {
	let (mut clock, mut handle) = Clock::new(
		Value::Fixed(ClockSpeed::SecondsPerTick(1.0)),
		fake_clock_id(),
	);
	handle.start();
	clock.on_start_processing();
	clock.update(
		1.5,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().ticks, 1);
	handle.pause();
	clock.on_start_processing();
	// the clock should not be ticking
	for _ in 0..3 {
		clock.update(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build(),
			&MockListenerInfoProviderBuilder::new(None, 0).build(),
		);
		clock.on_start_processing();
		assert!(!handle.ticking());
		assert_eq!(handle.time().ticks, 1);
	}
	handle.start();
	clock.on_start_processing();
	// make sure we've preserved the fractional position from before
	// pausing
	clock.update(
		0.4,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().ticks, 1);
	clock.update(
		0.1,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().ticks, 2);
}

/// Tests that a `Clock` can be stopped.
#[test]
fn stop() {
	let (mut clock, mut handle) = Clock::new(
		Value::Fixed(ClockSpeed::SecondsPerTick(1.0)),
		fake_clock_id(),
	);
	handle.start();
	clock.on_start_processing();
	clock.update(
		1.5,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	handle.stop();
	clock.on_start_processing();
	// the clock should not be ticking
	for _ in 0..3 {
		clock.update(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build(),
			&MockListenerInfoProviderBuilder::new(None, 0).build(),
		);
		clock.on_start_processing();
		assert!(!handle.ticking());
		assert_eq!(handle.time().ticks, 0);
	}
	handle.start();
	clock.on_start_processing();
	// make sure the fractional position has been reset
	clock.update(
		0.9,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().ticks, 0);
	clock.update(
		0.1,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().ticks, 1);
}

/// Tests that the speed of a [`Clock`] can be changed after creation.
#[test]
fn set_speed() {
	let (mut clock, mut handle) = Clock::new(
		Value::Fixed(ClockSpeed::SecondsPerTick(1.0)),
		fake_clock_id(),
	);
	handle.start();
	clock.on_start_processing();
	handle.set_speed(
		Value::Fixed(ClockSpeed::SecondsPerTick(0.5)),
		Tween {
			duration: Duration::ZERO,
			..Default::default()
		},
	);
	clock.on_start_processing();
	clock.update(
		1.0,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().ticks, 2);
	clock.update(
		1.0,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().ticks, 4);
}

/// Tests that a clock speed tween properly responds to ticks from
/// other clocks when the start time is set to a clock time.
#[test]
fn set_speed_with_clock_time_start() {
	let (clock_info_provider, clock_id) = {
		let mut builder = MockClockInfoProviderBuilder::new(1);
		let clock_id = builder.add(true, 0, 0.0).unwrap();
		(builder.build(), clock_id)
	};

	let (mut clock, mut handle) = Clock::new(
		Value::Fixed(ClockSpeed::SecondsPerTick(1.0)),
		fake_clock_id(),
	);
	handle.start();
	clock.on_start_processing();
	handle.set_speed(
		Value::Fixed(ClockSpeed::SecondsPerTick(0.5)),
		Tween {
			duration: Duration::ZERO,
			start_time: StartTime::ClockTime(ClockTime {
				clock: clock_id,
				ticks: 1,
				fraction: 0.0,
			}),
			..Default::default()
		},
	);
	clock.on_start_processing();

	clock.update(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().ticks, 1);
	clock.update(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().ticks, 2);

	let clock_info_provider = {
		let mut builder = MockClockInfoProviderBuilder::new(1);
		builder.add(true, 1, 0.0).unwrap();
		builder.build()
	};

	clock.update(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().ticks, 4);
	clock.update(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().ticks, 6);
}

/// Tests that a clock correctly reports its fractional position.
#[test]
fn fractional_position() {
	let (mut clock, mut handle) = Clock::new(
		Value::Fixed(ClockSpeed::SecondsPerTick(1.0)),
		fake_clock_id(),
	);
	assert_eq!(handle.time().fraction, 0.0);
	// the clock is not started yet, so the fractional position should remain at 0
	clock.update(
		1.0,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().fraction, 0.0);
	// start the clock
	handle.start();
	clock.on_start_processing();
	clock.update(
		0.5,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().fraction, 0.5);
	clock.update(
		0.75,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
		&MockListenerInfoProviderBuilder::new(None, 0).build(),
	);
	clock.on_start_processing();
	assert_eq!(handle.time().fraction, 0.25);
}

fn fake_clock_id() -> ClockId {
	let mut arena = Arena::new(1);
	let key = arena.insert(()).unwrap();
	ClockId(key)
}
