use crate::{clock::ClockTime, StartTime};

use super::Tween;

type JustFinishedTween = bool;

#[derive(Debug, Clone, Copy)]
enum State {
	Idle,
	Tweening {
		values: (f64, f64),
		time: f64,
		tween: Tween,
		waiting_to_start: bool,
	},
}

/// A value that can be smoothly transitioned to other values
/// using [`Tween`]s.
///
/// This is a utility for writing [`Sound`](crate::sound::Sound)s.
/// If you want to smoothly transition values from gameplay code,
/// consider using [parameters](crate::parameter).
#[derive(Debug, Clone, Copy)]
pub struct Tweenable {
	state: State,
	value: f64,
}

impl Tweenable {
	/// Creates a new [`Tweenable`] with an initial value.
	pub fn new(initial_value: f64) -> Self {
		Self {
			state: State::Idle,
			value: initial_value,
		}
	}

	/// Returns the current value of the [`Tweenable`].
	pub fn value(&self) -> f64 {
		self.value
	}

	/// Starts transitioning the [`Tweenable`] to the target
	/// value with the given tween.
	pub fn set(&mut self, target: f64, tween: Tween) {
		self.state = State::Tweening {
			values: (self.value, target),
			time: 0.0,
			tween,
			waiting_to_start: matches!(tween.start_time, StartTime::ClockTime(..)),
		}
	}

	/// Updates the [`Tweenable`] and returns `true` if it just finished
	/// a tween that was in progress.
	pub fn update(&mut self, dt: f64) -> JustFinishedTween {
		if let State::Tweening {
			values,
			time,
			tween,
			waiting_to_start,
		} = &mut self.state
		{
			if *waiting_to_start {
				return false;
			}
			*time += dt;
			if *time >= tween.duration.as_secs_f64() {
				self.value = values.1;
				self.state = State::Idle;
				return true;
			} else {
				self.value = values.0 + (values.1 - values.0) * tween.value(*time);
			}
		}
		false
	}

	pub fn on_clock_tick(&mut self, time: ClockTime) {
		if let State::Tweening {
			waiting_to_start,
			tween: Tween {
				start_time: StartTime::ClockTime(start_clock_time),
				..
			},
			..
		} = &mut self.state
		{
			if *waiting_to_start {
				if time.clock == start_clock_time.clock && time.ticks >= start_clock_time.ticks {
					*waiting_to_start = false;
				}
			}
		}
	}
}
