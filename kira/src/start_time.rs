use crate::clock::ClockId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StartTime {
	Immediate,
	ClockTime(ClockId, u64),
}

impl StartTime {
	pub fn clock_time(clock: impl Into<ClockId>, time: u64) -> Self {
		Self::ClockTime(clock.into(), time)
	}
}

impl Default for StartTime {
	fn default() -> Self {
		Self::Immediate
	}
}
