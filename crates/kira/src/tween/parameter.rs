#[cfg(test)]
mod test;

mod value;

use std::time::Duration;

pub use value::*;

use crate::{
	clock::clock_info::{ClockInfoProvider, WhenToStart},
	command::{CommandReader, ValueChangeCommand},
	modulator::value_provider::ModulatorValueProvider,
	tween::{Tween, Tweenable},
	StartTime,
};

/// Manages and updates a value that can be smoothly transitioned
/// and linked to modulators.
///
/// You'll only need to use this if you're creating your own
/// [`Sound`](crate::sound::Sound), [`Effect`](crate::track::effect::Effect),
/// or [`Modulator`](crate::modulator::Modulator) implementations. If you
/// want to adjust a parameter of something from gameplay code (such as the
/// volume of a sound or the speed of a clock), use the functions on that
/// object's handle.
#[derive(Clone)]
pub struct Parameter<T: Tweenable = f64> {
	state: State<T>,
	raw_value: T,
	stagnant: bool,
}

impl<T: Tweenable> Parameter<T> {
	/// Creates a new [`Parameter`] with an initial [`Value`].
	///
	/// The `default_raw_value` is used if the parameter is linked to a modulator
	/// that doesn't exist.
	pub fn new(initial_value: Value<T>, default_raw_value: T) -> Self {
		Self {
			state: State::Idle {
				value: initial_value,
			},
			raw_value: match initial_value {
				Value::Fixed(value) => value,
				Value::FromModulator { .. } => default_raw_value,
			},
			stagnant: matches!(initial_value, Value::Fixed(_)),
		}
	}

	/// Returns the current actual value of the parameter.
	pub fn value(&self) -> T {
		self.raw_value
	}

	/// Starts a transition from the current value to the target value.
	pub fn set(&mut self, target: Value<T>, tween: Tween) {
		self.stagnant = false;
		self.state = State::Tweening {
			start: self.value(),
			target,
			time: 0.0,
			tween,
		};
	}

	pub fn read_command(&mut self, command_reader: &mut CommandReader<ValueChangeCommand<T>>)
	where
		T: Send,
	{
		if let Some(ValueChangeCommand { target, tween }) = command_reader.read() {
			self.set(target, tween);
		}
	}

	/// Updates any in-progress transitions and keeps the value up-to-date
	/// with any linked modulators.
	///
	/// Returns `true` if a transition just finished after this update.
	pub fn update(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> ParameterUpdateInfo {
		if self.stagnant {
			return ParameterUpdateInfo::default();
		}
		let info = self.update_tween(dt, clock_info_provider);
		if let Some(raw_value) = self.calculate_new_raw_value(modulator_value_provider) {
			self.raw_value = raw_value;
		}
		info
	}

	fn update_tween(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
	) -> ParameterUpdateInfo {
		if let State::Tweening {
			target,
			time,
			tween,
			..
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
				return ParameterUpdateInfo::default();
			}
			*time += dt;
			let just_finished = if *time >= tween.duration.as_secs_f64() {
				if matches!(target, Value::Fixed(_)) {
					self.stagnant = true;
				}
				self.state = State::Idle { value: *target };
				true
			} else {
				false
			};
			return ParameterUpdateInfo {
				started,
				just_finished,
			};
		}
		ParameterUpdateInfo::default()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ParameterUpdateInfo {
	/// Whether the current tween (if any) has started yet. This will be `false`
	/// if the tween is waiting for a delay or clock time.
	pub started: bool,
	/// Whether the current tween just finished this update.
	pub just_finished: bool,
}
