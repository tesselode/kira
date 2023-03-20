mod builder;
mod command;
mod handle;

#[cfg(test)]
mod test;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

pub use builder::*;
use command::*;
pub use handle::*;
use ringbuf::HeapConsumer;

use crate::{
	clock::clock_info::{ClockInfoProvider, WhenToStart},
	tween::{Tween, Tweenable},
};

use super::Modulator;

struct Tweener {
	state: State,
	value: f64,
	command_consumer: HeapConsumer<Command>,
	shared: Arc<TweenerShared>,
}

impl Tweener {
	fn new(
		initial_value: f64,
		command_consumer: HeapConsumer<Command>,
		shared: Arc<TweenerShared>,
	) -> Self {
		Self {
			state: State::Idle,
			value: initial_value,
			command_consumer,
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
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::Set { target, tween } => self.set(target, tween),
			}
		}
	}

	fn update(&mut self, dt: f64, clock_info_provider: &ClockInfoProvider) {
		if let State::Tweening {
			values,
			time,
			tween,
		} = &mut self.state
		{
			if clock_info_provider.when_to_start(tween.start_time) == WhenToStart::Now {
				*time += dt;
				if *time >= tween.duration.as_secs_f64() {
					self.value = values.1;
					self.state = State::Idle;
				} else {
					self.value = Tweenable::interpolate(values.0, values.1, tween.value(*time));
				}
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
