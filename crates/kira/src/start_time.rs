use std::time::Duration;

use crate::clock::ClockTime;

/// Describes when an action should occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StartTime {
	/// The action should occur immediately.
	Immediate,
	/// The action should occur a certain amount of time from now.
	Delayed(Duration),
	/// The action should occur when a clock reaches a
	/// specific time.
	ClockTime(ClockTime),
}

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

impl Default for StartTime {
	fn default() -> Self {
		Self::Immediate
	}
}
