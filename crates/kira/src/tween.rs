//! Smooth interpolation between values.

mod parameter;
mod tweenable;

pub use parameter::*;
pub use tweenable::*;

use std::time::Duration;

use crate::start_time::StartTime;

/// Curves the motion of a [`Tween`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Easing {
	/// Maintains a constant speed for the duration of the [`Tween`].
	Linear,
	InQuad,
	OutQuad,
	InOutQuad,
	InCubic,
	OutCubic,
	InOutCubic,
	InQuart,
	OutQuart,
	InOutQuart,
	InQuint,
	OutQuint,
	InOutQuint,
}

impl Easing {
	pub(crate) fn apply(&self, x: f64) -> f64 {
		match self {
			Easing::Linear => x,
			Easing::InQuad => in_pow(x, 2),
			Easing::OutQuad => out_pow(x, 2),
			Easing::InOutQuad => in_out_pow(x, 2),
			Easing::InCubic => in_pow(x, 3),
			Easing::OutCubic => out_pow(x, 3),
			Easing::InOutCubic => in_out_pow(x, 3),
			Easing::InQuart => in_pow(x, 4),
			Easing::OutQuart => out_pow(x, 4),
			Easing::InOutQuart => in_out_pow(x, 4),
			Easing::InQuint => in_pow(x, 5),
			Easing::OutQuint => out_pow(x, 5),
			Easing::InOutQuint => in_out_pow(x, 5),
		}
	}
}

fn in_pow(x: f64, power: i32) -> f64 {
	x.powi(power)
}

fn out_pow(x: f64, power: i32) -> f64 {
	1.0 - in_pow(1.0 - x, power)
}

fn in_out_pow(mut x: f64, power: i32) -> f64 {
	x *= 2.0;
	if x < 1.0 {
		0.5 * in_pow(x, power)
	} else {
		x = 2.0 - x;
		0.5 * (1.0 - in_pow(x, power)) + 0.5
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
	/// The duration of the motion (in milliseconds).
	pub duration: u32,
	/// The curve of the motion.
	pub easing: Easing,
}

impl Tween {
	pub(super) fn value(&self, time: f64) -> f64 {
		self.easing
			.apply(time / Duration::from_millis(self.duration as u64).as_secs_f64())
	}
}

impl Default for Tween {
	fn default() -> Self {
		Self {
			start_time: StartTime::default(),
			duration: 10,
			easing: Easing::Linear,
		}
	}
}
