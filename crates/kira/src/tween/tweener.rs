#[cfg(test)]
mod test;

use crate::clock::clock_info::{ClockInfoProvider, WhenToStart};

use super::{Tween, Tweenable};

type JustFinishedTween = bool;

#[derive(Debug, Clone, Copy)]
enum State<T: Tweenable> {
	Idle,
	Tweening {
		values: (T, T),
		time: f64,
		tween: Tween,
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
		}
	}

	/// Updates the [`Tweenable`] and returns `true` if it just finished
	/// a tween that was in progress.
	pub fn update(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
	) -> JustFinishedTween {
		if let State::Tweening {
			values,
			time,
			tween,
		} = &mut self.state
		{
			if clock_info_provider.when_to_start(tween.start_time) != WhenToStart::Now {
				return false;
			}
			*time += dt;
			if *time >= tween.duration.as_secs_f64() {
				self.value = values.1;
				self.state = State::Idle;
				return true;
			} else {
				self.value = T::interpolate(values.0, values.1, tween.value(*time));
			}
		}
		false
	}
}
