use std::time::Duration;

use crate::{
	clock::{
		clock_info::{ClockInfoProvider, WhenToStart},
		ClockTime,
	},
	Trigger,
};

/// Describes when an action should occur.
#[derive(Debug, Clone, Default)]
pub enum StartTime {
	/// The action should occur immediately.
	#[default]
	Immediate,
	/// The action should occur a certain amount of time from now.
	Delayed(Duration),
	/// The action should occur when a clock reaches a
	/// specific time.
	ClockTime(ClockTime),
	/// The action should occur when a trigger is fired.
	Trigger(Trigger),
}

impl StartTime {
	pub(crate) fn update(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
	) -> WillNeverStart {
		match self {
			StartTime::Immediate => {}
			StartTime::Delayed(time_remaining) => {
				if time_remaining.is_zero() {
					*self = StartTime::Immediate;
				} else {
					*time_remaining = time_remaining.saturating_sub(Duration::from_secs_f64(dt));
				}
			}
			StartTime::ClockTime(clock_time) => {
				match clock_info_provider.when_to_start(*clock_time) {
					WhenToStart::Now => {
						*self = StartTime::Immediate;
					}
					WhenToStart::Later => {}
					WhenToStart::Never => return true,
				}
			}
			StartTime::Trigger(trigger) => {
				if trigger.fired() {
					*self = StartTime::Immediate;
				}
			}
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

impl From<Trigger> for StartTime {
	fn from(v: Trigger) -> Self {
		Self::Trigger(v)
	}
}

impl From<&Trigger> for StartTime {
	fn from(v: &Trigger) -> Self {
		Self::Trigger(v.clone())
	}
}
