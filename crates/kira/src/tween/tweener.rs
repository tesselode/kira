use crate::{clock::ClockTime, StartTime};

use super::{Tween, Tweenable};

type JustFinishedTween = bool;

#[derive(Debug, Clone, Copy)]
enum State<T: Tweenable> {
	Idle,
	Tweening {
		values: (T, T),
		time: f64,
		tween: Tween,
		waiting_to_start: bool,
	},
}

/// Holds a value and plays back tweens which smoothly
/// adjust that value.
#[derive(Debug, Clone, Copy)]
pub struct Tweener<T: Tweenable = f64> {
	state: State<T>,
	value: T,
}

impl<T: Tweenable> Tweener<T> {
	/// Creates a new [`Tweenable`] with an initial value.
	pub fn new(initial_value: T) -> Self {
		Self {
			state: State::Idle,
			value: initial_value,
		}
	}

	/// Returns the current value of the [`Tweenable`].
	pub fn value(&self) -> T {
		self.value
	}

	/// Starts transitioning the [`Tweenable`] to the target
	/// value with the given tween.
	pub fn set(&mut self, target: T, tween: Tween) {
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
				self.value = T::lerp(values.0, values.1, tween.value(*time));
			}
		}
		false
	}

	/// Informs the [`Tweener`] about a clock tick.
	///
	/// If the current tween's start time is set to a clock
	/// time, and that time has been reached, the tween will
	/// start playing.
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
