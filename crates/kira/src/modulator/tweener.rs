//! Smoothly transitions values to other values.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

#[cfg(test)]
mod test;

use std::{
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

use crate::{
	clock::clock_info::{ClockInfoProvider, WhenToStart},
	command_writers_and_readers,
	tween::{Tween, Tweenable},
	StartTime,
};

use super::{value_provider::ModulatorValueProvider, Modulator};

struct Tweener {
	state: State,
	value: f64,
	command_readers: CommandReaders,
	shared: Arc<TweenerShared>,
}

impl Tweener {
	fn new(
		initial_value: f64,
		command_readers: CommandReaders,
		shared: Arc<TweenerShared>,
	) -> Self {
		Self {
			state: State::Idle,
			value: initial_value,
			command_readers,
			shared,
		}
	}

	fn set(&mut self, target: f64, tween: Tween) {
		self.state = State::Tweening {
			values: (self.value, target),
			time: 0.0,
			tween,
		}
	}
}

impl Modulator for Tweener {
	fn on_start_processing(&mut self) {
		if let Some(SetCommand { target, tween }) = self.command_readers.set.read().copied() {
			self.set(target, tween);
		}
	}

	fn update(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		_modulator_value_provider: &ModulatorValueProvider,
	) {
		if let State::Tweening {
			values,
			time,
			tween,
		} = &mut self.state
		{
			let started = match &mut tween.start_time {
				StartTime::Immediate => true,
				StartTime::Delayed(time_remaining) => {
					if time_remaining.is_zero() {
						true
					} else {
						*time_remaining =
							time_remaining.saturating_sub(Duration::from_secs_f64(dt));
						false
					}
				}
				StartTime::ClockTime(clock_time) => {
					clock_info_provider.when_to_start(*clock_time) == WhenToStart::Now
				}
			};
			if !started {
				return;
			}
			*time += dt;
			if *time >= tween.duration.as_secs_f64() {
				self.value = values.1;
				self.state = State::Idle;
			} else {
				self.value = Tweenable::interpolate(values.0, values.1, tween.value(*time));
			}
		}
	}

	fn value(&self) -> f64 {
		self.value
	}

	fn finished(&self) -> bool {
		self.shared.removed.load(Ordering::SeqCst)
	}
}

enum State {
	Idle,
	Tweening {
		values: (f64, f64),
		time: f64,
		tween: Tween,
	},
}

struct TweenerShared {
	removed: AtomicBool,
}

impl TweenerShared {
	fn new() -> Self {
		Self {
			removed: AtomicBool::new(false),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SetCommand {
	target: f64,
	tween: Tween,
}

command_writers_and_readers!(
	struct {
		set: SetCommand
	}
);
