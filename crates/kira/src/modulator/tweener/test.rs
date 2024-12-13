use std::time::Duration;

use crate::arena::Arena;

use crate::clock::ClockInfo;
use crate::info::MockInfoBuilder;
use crate::{
	clock::ClockTime,
	modulator::{tweener::TweenerBuilder, ModulatorBuilder, ModulatorId},
	tween::Tween,
	StartTime,
};

/// Tests that the basic tweening behavior of a `Tweener`
/// works properly.
#[test]
#[allow(clippy::float_cmp)]
fn tweening() {
	let (mut tweener, mut handle) =
		TweenerBuilder { initial_value: 0.0 }.build(generate_fake_modulator_id());
	let info = MockInfoBuilder::new().build();
	let info = info.for_single_frame(0);

	// value should not be changing yet
	for _ in 0..3 {
		tweener.update(1.0, &info);
		assert_eq!(tweener.value(), 0.0);
	}

	handle.set(
		1.0,
		Tween {
			duration: Duration::from_secs(2),
			..Default::default()
		},
	);
	tweener.on_start_processing();

	tweener.update(1.0, &info);
	assert_eq!(tweener.value(), 0.5);
	tweener.update(1.0, &info);
	assert_eq!(tweener.value(), 1.0);
	tweener.update(1.0, &info);
	assert_eq!(tweener.value(), 1.0);
}

/// Tests that a `Tweener` with a delayed start time waits for
/// that time before it begins tweening.
#[test]
#[allow(clippy::float_cmp)]
fn waits_for_delay() {
	let info = MockInfoBuilder::new().build();
	let info = info.for_single_frame(0);

	let (mut tweener, mut handle) =
		TweenerBuilder { initial_value: 0.0 }.build(generate_fake_modulator_id());
	handle.set(
		1.0,
		Tween {
			start_time: StartTime::Delayed(Duration::from_secs(2)),
			duration: Duration::from_secs(1),
			..Default::default()
		},
	);
	tweener.on_start_processing();

	// value should not be changing yet
	for _ in 0..2 {
		assert_eq!(tweener.value(), 0.0);
		tweener.update(1.0, &info);
	}

	// the tween should start now
	tweener.update(1.0, &info);
	assert_eq!(tweener.value(), 1.0);
}

/// Tests that a `Tweener` with a clock time set as
/// the start time waits for that time before it
/// begins tweening.
#[test]
#[allow(clippy::float_cmp)]
fn waits_for_start_time() {
	let (mut tweener, mut handle) =
		TweenerBuilder { initial_value: 0.0 }.build(generate_fake_modulator_id());
	let mut info_builder = MockInfoBuilder::new();
	let clock_id_1 = info_builder.add_clock(vec![ClockInfo {
		ticking: true,
		ticks: 0,
		fraction: 0.0,
	}]);
	let info = info_builder.build();
	let info = info.for_single_frame(0);

	handle.set(
		1.0,
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
	tweener.on_start_processing();

	// value should not be changing yet
	for _ in 0..3 {
		tweener.update(1.0, &info);
		assert_eq!(tweener.value(), 0.0);
	}

	let info = {
		let mut builder = MockInfoBuilder::new();
		builder.add_clock(vec![ClockInfo {
			ticking: true,
			ticks: 1,
			fraction: 0.0,
		}]);
		builder.add_clock(vec![ClockInfo {
			ticking: true,
			ticks: 0,
			fraction: 0.0,
		}]);
		builder.build()
	};
	let info = info.for_single_frame(0);

	// the tween is set to start at tick 2, so it should not
	// start yet
	for _ in 0..3 {
		tweener.update(1.0, &info);
		assert_eq!(tweener.value(), 0.0);
	}

	let info = {
		let mut builder = MockInfoBuilder::new();
		builder.add_clock(vec![ClockInfo {
			ticking: true,
			ticks: 1,
			fraction: 0.0,
		}]);
		builder.add_clock(vec![ClockInfo {
			ticking: true,
			ticks: 2,
			fraction: 0.0,
		}]);
		builder.build()
	};
	let info = info.for_single_frame(0);

	// a different clock reached tick 2, so the tween should not
	// start yet
	for _ in 0..3 {
		tweener.update(1.0, &info);
		assert_eq!(tweener.value(), 0.0);
	}

	let info = {
		let mut builder = MockInfoBuilder::new();
		builder.add_clock(vec![ClockInfo {
			ticking: true,
			ticks: 2,
			fraction: 0.0,
		}]);
		builder.add_clock(vec![ClockInfo {
			ticking: true,
			ticks: 2,
			fraction: 0.0,
		}]);
		builder.build()
	};
	let info = info.for_single_frame(0);

	// the tween should start now
	tweener.update(1.0, &info);
	assert_eq!(tweener.value(), 1.0);
}

fn generate_fake_modulator_id() -> ModulatorId {
	let arena = Arena::<()>::new(1);
	ModulatorId(arena.controller().try_reserve().unwrap())
}
