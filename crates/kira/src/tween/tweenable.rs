use crate::{
	clock::{ClockTime, Clocks},
	StartTime,
};

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

#[derive(Debug, Clone, Copy)]
pub struct Tweenable {
	state: State,
	value: f64,
}

impl Tweenable {
	pub fn new(initial_value: f64) -> Self {
		Self {
			state: State::Idle,
			value: initial_value,
		}
	}

	pub fn value(&self) -> f64 {
		self.value
	}

	pub fn set(&mut self, target: f64, tween: Tween) {
		self.state = State::Tweening {
			values: (self.value, target),
			time: 0.0,
			tween,
			waiting_to_start: matches!(tween.start_time, StartTime::ClockTime(..)),
		}
	}

	pub fn update(&mut self, dt: f64, clocks: &Clocks) -> JustFinishedTween {
		if let State::Tweening {
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
				self.value = values.1;
				self.state = State::Idle;
				return true;
			} else {
				self.value = values.0 + (values.1 - values.0) * tween.value(*time);
			}
		}
		false
	}
}
