use crate::clock::ClockId;

/// Describes when an action should occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StartTime {
	/// The action should occur immediately.
	Immediate,
	/// The action should occur when a clock reaches a
	/// specific time.
	ClockTime(ClockId, u64),
}

impl StartTime {
	/// A helper for creating [`StartTime::ClockTime`]s.
	pub fn clock_time(clock: impl Into<ClockId>, time: u64) -> Self {
		Self::ClockTime(clock.into(), time)
	}
}

impl Default for StartTime {
	fn default() -> Self {
		Self::Immediate
	}
}
