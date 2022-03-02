use std::time::Duration;

use crate::{
	clock::ClockTime,
	manager::{backend::mock::MockBackend, AudioManager},
	tween::Tween,
	ClockSpeed, StartTime,
};

use super::Clock;

/// Tests that a `Clock` is stopped when it's first created.
#[test]
fn initially_stopped() {
	let mut clock = Clock::new(ClockSpeed::SecondsPerTick(1.0));
	let shared = clock.shared();
	for _ in 0..3 {
		assert!(!shared.ticking());
		assert_eq!(shared.ticks(), 0);
		clock.update(1.0);
	}
}

/// Tests that a `Clock` ticks.
#[test]
fn basic_behavior() {
	let mut clock = Clock::new(ClockSpeed::SecondsPerTick(1.0));
	let shared = clock.shared();
	clock.start();
	for i in 0..3 {
		assert!(shared.ticking());
		assert_eq!(shared.ticks(), i);
		assert_eq!(clock.update(1.0), Some(i + 1));
	}
}

/// Tests that a `Clock` can be paused.
#[test]
fn pause() {
	let mut clock = Clock::new(ClockSpeed::SecondsPerTick(1.0));
	let shared = clock.shared();
	clock.start();
	clock.update(1.5);
	assert_eq!(shared.ticks(), 1);
	clock.pause();
	// the clock should not be ticking
	for _ in 0..3 {
		assert!(!shared.ticking());
		assert_eq!(shared.ticks(), 1);
	}
	clock.start();
	// make sure we've preserved the fractional position from before
	// pausing
	clock.update(0.4);
	assert_eq!(shared.ticks(), 1);
	clock.update(0.1);
	assert_eq!(shared.ticks(), 2);
}

/// Tests that a `Clock` can be stopped.
#[test]
fn stop() {
	let mut clock = Clock::new(ClockSpeed::SecondsPerTick(1.0));
	let shared = clock.shared();
	clock.start();
	clock.update(1.5);
	clock.stop();
	// the clock should not be ticking
	for _ in 0..3 {
		assert!(!shared.ticking());
		assert_eq!(shared.ticks(), 0);
	}
	clock.start();
	// make sure the fractional position has been reset
	clock.update(0.9);
	assert_eq!(shared.ticks(), 0);
	clock.update(0.1);
	assert_eq!(shared.ticks(), 1);
}

/// Tests that the speed of a [`Clock`] can be changed after creation.
#[test]
fn set_speed() {
	let mut clock = Clock::new(ClockSpeed::SecondsPerTick(1.0));
	let shared = clock.shared();
	clock.start();
	clock.set_speed(
		ClockSpeed::SecondsPerTick(0.5),
		Tween {
			duration: Duration::ZERO,
			..Default::default()
		},
	);
	clock.update(1.0);
	assert_eq!(shared.ticks(), 2);
	clock.update(1.0);
	assert_eq!(shared.ticks(), 4);
}

/// Tests that a clock speed tween properly responds to ticks from
/// other clocks when the start time is set to a clock time.
#[test]
fn set_speed_with_clock_time_start() {
	let mut manager = AudioManager::<MockBackend>::new(Default::default()).unwrap();
	let other_clock = manager.add_clock(ClockSpeed::SecondsPerTick(1.0)).unwrap();
	let mut clock = Clock::new(ClockSpeed::SecondsPerTick(1.0));
	let shared = clock.shared();
	clock.start();
	clock.set_speed(
		ClockSpeed::SecondsPerTick(0.5),
		Tween {
			duration: Duration::ZERO,
			start_time: StartTime::ClockTime(other_clock.time()),
			..Default::default()
		},
	);
	clock.update(1.0);
	assert_eq!(shared.ticks(), 1);
	clock.update(1.0);
	assert_eq!(shared.ticks(), 2);
	clock.on_clock_tick(ClockTime {
		clock: other_clock.id(),
		ticks: 0,
	});
	clock.update(1.0);
	assert_eq!(shared.ticks(), 4);
	clock.update(1.0);
	assert_eq!(shared.ticks(), 6);
}
