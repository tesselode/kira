#[cfg(test)]
mod test;

mod value;

pub use value::*;

use crate::{
	clock::clock_info::{ClockInfoProvider, WhenToStart},
	modulator::value_provider::ModulatorValueProvider,
	tween::{Tween, Tweenable},
};

#[derive(Clone)]
pub struct Parameter<T: Tweenable = f64> {
	state: State<T>,
	raw_value: T,
}

impl<T: Tweenable> Parameter<T> {
	pub fn new(initial_value: Value<T>, default_raw_value: T) -> Self {
		Self {
			state: State::Idle {
				value: initial_value,
			},
			raw_value: match initial_value {
				Value::Fixed(value) => value,
				Value::FromModulator { .. } => default_raw_value,
			},
		}
	}

	pub fn value(&self) -> T {
		self.raw_value
	}

	pub fn set(&mut self, target: Value<T>, tween: Tween) {
		self.state = State::Tweening {
			start: self.value(),
			target,
			time: 0.0,
			tween,
		};
	}

	pub fn update(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> JustFinishedTween {
		let just_finished_tween = self.update_tween(dt, clock_info_provider);
		if let Some(raw_value) = self.calculate_new_raw_value(modulator_value_provider) {
			self.raw_value = raw_value;
		}
		just_finished_tween
	}

	fn update_tween(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
	) -> JustFinishedTween {
		if let State::Tweening {
			target,
			time,
			tween,
			..
		} = &mut self.state
		{
			if clock_info_provider.when_to_start(tween.start_time) != WhenToStart::Now {
				return false;
			}
			*time += dt;
			if *time >= tween.duration.as_secs_f64() {
				self.state = State::Idle { value: *target };
				return true;
			}
		}
		false
	}

	fn calculate_new_raw_value(
		&self,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Option<T> {
		match &self.state {
			State::Idle { value } => value.raw_value(modulator_value_provider),
			State::Tweening {
				start,
				target,
				time,
				tween,
			} => {
				if tween.duration.is_zero() {
					return None;
				}
				target
					.raw_value(modulator_value_provider)
					.map(|target| T::interpolate(*start, target, tween.value(*time)))
			}
		}
	}
}

#[derive(Clone)]
enum State<T: Tweenable> {
	Idle {
		value: Value<T>,
	},
	Tweening {
		start: T,
		target: Value<T>,
		time: f64,
		tween: Tween,
	},
}

type JustFinishedTween = bool;
