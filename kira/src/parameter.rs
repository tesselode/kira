mod handle;
mod tween;

use std::{
	ops::RangeInclusive,
	sync::{
		atomic::{AtomicBool, AtomicU64, Ordering},
		Arc,
	},
};

use atomic_arena::Index;

use crate::{clock::ClockTime, manager::resources::clocks::Clocks, start_time::StartTime};

pub use handle::*;
pub use tween::*;

type JustFinishedTween = bool;

/// A unique identifier for a parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParameterId(pub(crate) Index);

pub(crate) struct ParameterShared {
	value: AtomicU64,
	paused: AtomicBool,
	removed: AtomicBool,
}

impl ParameterShared {
	pub fn new(value: f64) -> Self {
		Self {
			value: AtomicU64::new(value.to_bits()),
			paused: AtomicBool::new(false),
			removed: AtomicBool::new(false),
		}
	}

	pub fn value(&self) -> f64 {
		f64::from_bits(self.value.load(Ordering::SeqCst))
	}

	pub fn paused(&self) -> bool {
		self.paused.load(Ordering::SeqCst)
	}

	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

enum ParameterState {
	Idle,
	Tweening {
		values: RangeInclusive<f64>,
		time: f64,
		tween: Tween,
		waiting_to_start: bool,
	},
}

pub(crate) struct Parameter {
	state: ParameterState,
	paused: bool,
	value: f64,
	shared: Arc<ParameterShared>,
}

impl Parameter {
	pub fn new(value: f64) -> Self {
		Self {
			state: ParameterState::Idle,
			paused: false,
			value,
			shared: Arc::new(ParameterShared::new(value)),
		}
	}

	pub fn shared(&self) -> Arc<ParameterShared> {
		self.shared.clone()
	}

	pub fn value(&self) -> f64 {
		self.value
	}

	pub fn pause(&mut self) {
		self.paused = true;
		self.shared.paused.store(true, Ordering::SeqCst);
	}

	pub fn resume(&mut self) {
		self.paused = false;
		self.shared.paused.store(false, Ordering::SeqCst);
	}

	pub fn set(&mut self, target: f64, tween: Tween) {
		self.state = ParameterState::Tweening {
			values: self.value..=target,
			time: 0.0,
			tween,
			waiting_to_start: matches!(tween.start_time, StartTime::ClockTime(..)),
		};
	}

	pub fn on_start_processing(&self) {
		self.shared
			.value
			.store(self.value.to_bits(), Ordering::SeqCst);
	}

	pub fn update(&mut self, dt: f64, clocks: &Clocks) -> JustFinishedTween {
		if self.paused {
			return false;
		}
		if let ParameterState::Tweening {
			values,
			time,
			tween,
			waiting_to_start,
		} = &mut self.state
		{
			if *waiting_to_start {
				if let StartTime::ClockTime(ClockTime { clock, ticks }) = tween.start_time {
					if let Some(clock) = clocks.get(clock) {
						if clock.ticking() && clock.ticks() >= ticks {
							*waiting_to_start = false;
						}
					}
				} else {
					panic!(
						"waiting_to_start should always be false if the start_time is Immediate"
					);
				}
			}
			if *waiting_to_start {
				return false;
			}
			*time += dt;
			if *time >= tween.duration.as_secs_f64() {
				self.value = *values.end();
				self.state = ParameterState::Idle;
				return true;
			} else {
				self.value = values.start() + (values.end() - values.start()) * tween.value(*time);
			}
		}
		false
	}
}
