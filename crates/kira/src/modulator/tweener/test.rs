use std::time::Duration;

use crate::arena::Arena;

use crate::{
	clock::{clock_info::MockClockInfoProviderBuilder, ClockTime},
	modulator::{
		tweener::TweenerBuilder, value_provider::MockModulatorValueProviderBuilder,
		ModulatorBuilder, ModulatorId,
	},
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
	let clock_info_provider = MockClockInfoProviderBuilder::new(1).build();
	let modulator_value_provider = MockModulatorValueProviderBuilder::new(0).build();

	// value should not be changing yet
	for _ in 0..3 {
		tweener.update(1.0, &clock_info_provider, &modulator_value_provider);
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

	tweener.update(1.0, &clock_info_provider, &modulator_value_provider);
	assert_eq!(tweener.value(), 0.5);
	tweener.update(1.0, &clock_info_provider, &modulator_value_provider);
	assert_eq!(tweener.value(), 1.0);
	tweener.update(1.0, &clock_info_provider, &modulator_value_provider);
	assert_eq!(tweener.value(), 1.0);
}

/// Tests that a `Tweener` with a delayed start time waits for
/// that time before it begins tweening.
#[test]
#[allow(clippy::float_cmp)]
fn waits_for_delay() {
	let clock_info_provider = MockClockInfoProviderBuilder::new(0).build();
	let modulator_value_provider = MockModulatorValueProviderBuilder::new(0).build();

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
		tweener.update(1.0, &clock_info_provider, &modulator_value_provider);
	}

	// the tween should start now
	tweener.update(1.0, &clock_info_provider, &modulator_value_provider);
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
	let (clock_info_provider, clock_id_1) = {
		let mut builder = MockClockInfoProviderBuilder::new(2);
		let clock_id_1 = builder.add(true, 0, 0.0).unwrap();
		builder.add(true, 0, 0.0).unwrap();
		(builder.build(), clock_id_1)
	};
	let modulator_value_provider = MockModulatorValueProviderBuilder::new(0).build();

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
		tweener.update(1.0, &clock_info_provider, &modulator_value_provider);
		assert_eq!(tweener.value(), 0.0);
	}

	let clock_info_provider = {
		let mut builder = MockClockInfoProviderBuilder::new(2);
		builder.add(true, 1, 0.0).unwrap();
		builder.add(true, 0, 0.0).unwrap();
		builder.build()
	};

	// the tween is set to start at tick 2, so it should not
	// start yet
	for _ in 0..3 {
		tweener.update(1.0, &clock_info_provider, &modulator_value_provider);
		assert_eq!(tweener.value(), 0.0);
	}

	let clock_info_provider = {
		let mut builder = MockClockInfoProviderBuilder::new(2);
		builder.add(true, 1, 0.0).unwrap();
		builder.add(true, 2, 0.0).unwrap();
		builder.build()
	};

	// a different clock reached tick 2, so the tween should not
	// start yet
	for _ in 0..3 {
		tweener.update(1.0, &clock_info_provider, &modulator_value_provider);
		assert_eq!(tweener.value(), 0.0);
	}

	let clock_info_provider = {
		let mut builder = MockClockInfoProviderBuilder::new(2);
		builder.add(true, 2, 0.0).unwrap();
		builder.add(true, 2, 0.0).unwrap();
		builder.build()
	};

	// the tween should start now
	tweener.update(1.0, &clock_info_provider, &modulator_value_provider);
	assert_eq!(tweener.value(), 1.0);
}

fn generate_fake_modulator_id() -> ModulatorId {
	let arena = Arena::<()>::new(1);
	ModulatorId(arena.controller().try_reserve().unwrap())
}
