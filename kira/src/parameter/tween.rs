use std::time::Duration;

use crate::start_time::StartTime;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
	Linear,
	InPowi(i32),
	OutPowi(i32),
	InOutPowi(i32),
	InPowf(f64),
	OutPowf(f64),
	InOutPowf(f64),
}

impl Easing {
	fn apply(&self, mut x: f64) -> f64 {
		match self {
			Easing::Linear => x,
			Easing::InPowi(power) => x.powi(*power),
			Easing::OutPowi(power) => 1.0 - Self::InPowi(*power).apply(1.0 - x),
			Easing::InOutPowi(power) => {
				x *= 2.0;
				if x < 1.0 {
					0.5 * Self::InPowi(*power).apply(x)
				} else {
					x = 2.0 - x;
					0.5 * (1.0 - Self::InPowi(*power).apply(x)) + 0.5
				}
			}
			Easing::InPowf(power) => x.powf(*power),
			Easing::OutPowf(power) => 1.0 - Self::InPowf(*power).apply(1.0 - x),
			Easing::InOutPowf(power) => {
				x *= 2.0;
				if x < 1.0 {
					0.5 * Self::InPowf(*power).apply(x)
				} else {
					x = 2.0 - x;
					0.5 * (1.0 - Self::InPowf(*power).apply(x)) + 0.5
				}
			}
		}
	}
}

impl Default for Easing {
	fn default() -> Self {
		Self::Linear
	}
}

/// A movement of one value to another over time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tween {
	/// When the motion starts.
	pub start_time: StartTime,
	/// The duration of the motion.
	pub duration: Duration,
	/// The curve of the motion.
	pub easing: Easing,
}

impl Tween {
	pub(super) fn value(&self, time: f64) -> f64 {
		self.easing.apply(time / self.duration.as_secs_f64())
	}
}

impl Default for Tween {
	fn default() -> Self {
		Self {
			start_time: StartTime::default(),
			duration: Duration::from_millis(10),
			easing: Easing::Linear,
		}
	}
}
