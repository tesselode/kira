use std::time::Duration;

use crate::{
	clock::ClockTime,
	info::{SingleFrameInfo, WhenToStart},
};

/// Describes when an action should occur.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum StartTime {
	/// The action should occur immediately.
	#[default]
	Immediate,
	/// The action should occur a certain amount of time from now.
	Delayed(Duration),
	/// The action should occur when a clock reaches a
	/// specific time.
	ClockTime(ClockTime),
}

impl StartTime {
	pub(crate) fn update(&mut self, dt: f64, info: &SingleFrameInfo) -> WillNeverStart {
		match self {
			StartTime::Immediate => {}
			StartTime::Delayed(time_remaining) => {
				*time_remaining = time_remaining.saturating_sub(Duration::from_secs_f64(dt));
				if time_remaining.is_zero() {
					*self = StartTime::Immediate;
				}
			}
			StartTime::ClockTime(clock_time) => match info.when_to_start(*clock_time) {
				WhenToStart::Now => {
					*self = StartTime::Immediate;
				}
				WhenToStart::Later => {}
				WhenToStart::Never => return true,
			},
		}
		false
	}
}

pub(crate) type WillNeverStart = bool;

impl From<Duration> for StartTime {
	fn from(v: Duration) -> Self {
		Self::Delayed(v)
	}
}

impl From<ClockTime> for StartTime {
	fn from(v: ClockTime) -> Self {
		Self::ClockTime(v)
	}
}
