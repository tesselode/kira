use std::time::Duration;

use atomic_arena::Arena;

use crate::{
	clock::{ClockId, ClockTime},
	tween::Tween,
	StartTime,
};

use super::Tweener;

/// Tests that the basic tweening behavior of a `Tweener`
/// works properly.
#[test]
#[allow(clippy::float_cmp)]
fn tweening() {
	let mut tweener = Tweener::new(0.0);

	// value should not be changing yet
	for _ in 0..3 {
		assert_eq!(tweener.value(), 0.0);
		assert!(!tweener.update(1.0));
	}

	tweener.set(
		1.0,
		Tween {
			duration: Duration::from_secs(2),
			..Default::default()
		},
	);

	assert!(!tweener.update(1.0));
	assert_eq!(tweener.value(), 0.5);
	assert!(tweener.update(1.0));
	assert_eq!(tweener.value(), 1.0);
	assert!(!tweener.update(1.0));
	assert_eq!(tweener.value(), 1.0);
}

/// Tests that a Tweener with a clock time set as
/// the start time waits for that time before it
/// begins tweening.
#[test]
#[allow(clippy::float_cmp)]
fn waits_for_start_time() {
	// create some fake ClockIds
	let mut dummy_arena = Arena::new(2);
	let key1 = dummy_arena.insert(()).unwrap();
	let key2 = dummy_arena.insert(()).unwrap();
	let clock_id_1 = ClockId(key1);
	let clock_id_2 = ClockId(key2);

	let mut tweener = Tweener::new(0.0);
	tweener.set(
		1.0,
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
		assert_eq!(tweener.value(), 0.0);
		assert!(!tweener.update(1.0));
	}

	// the tween is set to start at tick 2, so it should not
	// start yet
	tweener.on_clock_tick(ClockTime {
		clock: clock_id_1,
		ticks: 1,
	});
	for _ in 0..3 {
		assert_eq!(tweener.value(), 0.0);
		assert!(!tweener.update(1.0));
	}

	// this is a tick event for a different clock, so the
	// tween should not start yet
	tweener.on_clock_tick(ClockTime {
		clock: clock_id_2,
		ticks: 2,
	});
	for _ in 0..3 {
		assert_eq!(tweener.value(), 0.0);
		assert!(!tweener.update(1.0));
	}

	// the tween should start now
	tweener.on_clock_tick(ClockTime {
		clock: clock_id_1,
		ticks: 2,
	});
	assert!(tweener.update(1.0));
	assert_eq!(tweener.value(), 1.0);
}
