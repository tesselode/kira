use std::cmp::Ordering;

use approx::assert_relative_eq;
use atomic_arena::Arena;

use crate::clock::{ClockId, ClockTime};

#[test]
fn from_ticks_u64() {
	let id = fake_clock_id();
	assert_eq!(
		ClockTime::from_ticks_u64(id, 3),
		ClockTime {
			clock: id,
			ticks: 3,
			fraction: 0.0
		}
	)
}

#[test]
fn from_ticks_f64() {
	let id = fake_clock_id();
	assert_eq!(
		ClockTime::from_ticks_f64(id, 3.5),
		ClockTime {
			clock: id,
			ticks: 3,
			fraction: 0.5
		}
	)
}

#[test]
fn add_u64() {
	let id = fake_clock_id();
	assert_eq!(
		ClockTime {
			clock: id,
			ticks: 2,
			fraction: 0.0,
		} + 3,
		ClockTime {
			clock: id,
			ticks: 5,
			fraction: 0.0,
		}
	);
	assert_eq!(
		ClockTime {
			clock: id,
			ticks: 2,
			fraction: 0.5,
		} + 2,
		ClockTime {
			clock: id,
			ticks: 4,
			fraction: 0.5,
		}
	);
}

#[test]
fn sub_u64() {
	let id = fake_clock_id();
	assert_eq!(
		ClockTime {
			clock: id,
			ticks: 5,
			fraction: 0.0,
		} - 3,
		ClockTime {
			clock: id,
			ticks: 2,
			fraction: 0.0,
		}
	);
	assert_eq!(
		ClockTime {
			clock: id,
			ticks: 4,
			fraction: 0.5,
		} - 2,
		ClockTime {
			clock: id,
			ticks: 2,
			fraction: 0.5,
		}
	);
}

#[test]
fn add_f64() {
	let id = fake_clock_id();
	let time = ClockTime {
		clock: id,
		ticks: 2,
		fraction: 0.5,
	} + 3.7;
	assert_eq!(time.ticks, 6);
	assert_relative_eq!(time.fraction, 0.2);
	let time = ClockTime {
		clock: id,
		ticks: 2,
		fraction: 0.5,
	} + 3.5;
	assert_eq!(time.ticks, 6);
	assert_relative_eq!(time.fraction, 0.0);
	let id = fake_clock_id();
	let time = ClockTime {
		clock: id,
		ticks: 5,
		fraction: 0.5,
	} + (-3.7);
	assert_eq!(time.ticks, 1);
	assert_relative_eq!(time.fraction, 0.8);
}

#[test]
fn sub_f64() {
	let id = fake_clock_id();
	let time = ClockTime {
		clock: id,
		ticks: 5,
		fraction: 0.5,
	} - 3.7;
	assert_eq!(time.ticks, 1);
	assert_relative_eq!(time.fraction, 0.8);
	let time = ClockTime {
		clock: id,
		ticks: 5,
		fraction: 0.5,
	} - 3.5;
	assert_eq!(time.ticks, 2);
	assert_relative_eq!(time.fraction, 0.0);
	let id = fake_clock_id();
	let time = ClockTime {
		clock: id,
		ticks: 2,
		fraction: 0.5,
	} - (-3.7);
	assert_eq!(time.ticks, 6);
	assert_relative_eq!(time.fraction, 0.2);
}

#[test]
fn partial_cmp() {
	let ids = fake_clock_ids(2);
	assert_eq!(
		ClockTime {
			clock: ids[0],
			ticks: 5,
			fraction: 0.0
		}
		.partial_cmp(&ClockTime {
			clock: ids[0],
			ticks: 5,
			fraction: 0.0
		}),
		Some(Ordering::Equal)
	);
	assert_eq!(
		ClockTime {
			clock: ids[0],
			ticks: 4,
			fraction: 0.0
		}
		.partial_cmp(&ClockTime {
			clock: ids[0],
			ticks: 5,
			fraction: 0.0
		}),
		Some(Ordering::Less)
	);
	assert_eq!(
		ClockTime {
			clock: ids[0],
			ticks: 6,
			fraction: 0.0
		}
		.partial_cmp(&ClockTime {
			clock: ids[0],
			ticks: 5,
			fraction: 0.0
		}),
		Some(Ordering::Greater)
	);
	assert_eq!(
		ClockTime {
			clock: ids[0],
			ticks: 5,
			fraction: 0.0
		}
		.partial_cmp(&ClockTime {
			clock: ids[0],
			ticks: 5,
			fraction: 0.5
		}),
		Some(Ordering::Less)
	);
	assert_eq!(
		ClockTime {
			clock: ids[0],
			ticks: 5,
			fraction: 0.5
		}
		.partial_cmp(&ClockTime {
			clock: ids[0],
			ticks: 5,
			fraction: 0.0
		}),
		Some(Ordering::Greater)
	);
	assert_eq!(
		ClockTime {
			clock: ids[0],
			ticks: 5,
			fraction: 0.5
		}
		.partial_cmp(&ClockTime {
			clock: ids[1],
			ticks: 5,
			fraction: 0.0
		}),
		None
	);
	assert_eq!(
		ClockTime {
			clock: ids[0],
			ticks: 5,
			fraction: 0.5
		}
		.partial_cmp(&ClockTime {
			clock: ids[0],
			ticks: 5,
			fraction: f64::NAN,
		}),
		None
	);
}

fn fake_clock_id() -> ClockId {
	let mut arena = Arena::new(1);
	let key = arena.insert(()).unwrap();
	ClockId(key)
}

fn fake_clock_ids(count: usize) -> Vec<ClockId> {
	let mut arena = Arena::new(count);
	(0..count)
		.map(|_| arena.insert(()).unwrap())
		.map(ClockId)
		.collect()
}
