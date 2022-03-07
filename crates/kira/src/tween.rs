//! Smooth interpolation between values.

mod tweenable;
mod tweener;

pub use tweenable::*;
pub use tweener::*;

use std::time::Duration;

use crate::start_time::StartTime;

/// Curves the motion of a [`Tween`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Easing {
	/// Maintains a constant speed for the duration of the [`Tween`].
	Linear,
	/// Causes the [`Tween`] to start slow and speed up. A higher
	/// value causes the [`Tween`] to speed up more dramatically.
	InPowi(i32),
	/// Causes the [`Tween`] to start fast and slow down. A higher
	/// value causes the [`Tween`] to slow down more dramatically.
	OutPowi(i32),
	/// Causes the [`Tween`] to start slow, speed up, and then slow
	/// back down. A higher values causes the [`Tween`] to have more
	/// dramatic speed changes.
	InOutPowi(i32),
	/// Causes the [`Tween`] to start slow and speed up. A higher
	/// value causes the [`Tween`] to speed up more dramatically.
	///
	/// This is similar to [`InPowi`](Easing::InPowi), but allows
	/// for float intensity values at the cost of being more
	/// CPU intensive.
	InPowf(f64),
	/// Causes the [`Tween`] to start fast and slow down. A higher
	/// value causes the [`Tween`] to slow down more dramatically.
	///
	/// This is similar to [`OutPowi`](Easing::InPowi), but allows
	/// for float intensity values at the cost of being more
	/// CPU intensive.
	OutPowf(f64),
	/// Causes the [`Tween`] to start slow, speed up, and then slow
	/// back down. A higher values causes the [`Tween`] to have more
	/// dramatic speed changes.
	///
	/// This is similar to [`InOutPowi`](Easing::InPowi), but allows
	/// for float intensity values at the cost of being more
	/// CPU intensive.
	InOutPowf(f64),
}

impl Easing {
	pub(crate) fn apply(&self, mut x: f64) -> f64 {
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

/// Describes a smooth transition between values.
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
