use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::ClockId;

/// An instant in time associated with a [`Clock`](super::Clock).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockTime {
	/// The [`Clock`](super::Clock) this time is associated with.
	pub clock: ClockId,
	/// The elapsed time in ticks.
	pub ticks: u64,
}

impl Add<u64> for ClockTime {
	type Output = ClockTime;

	fn add(self, ticks: u64) -> Self::Output {
		Self {
			clock: self.clock,
			ticks: self.ticks + ticks,
		}
	}
}

impl AddAssign<u64> for ClockTime {
	fn add_assign(&mut self, ticks: u64) {
		self.ticks += ticks;
	}
}

impl Sub<u64> for ClockTime {
	type Output = ClockTime;

	fn sub(self, ticks: u64) -> Self::Output {
		Self {
			clock: self.clock,
			ticks: self.ticks - ticks,
		}
	}
}

impl SubAssign<u64> for ClockTime {
	fn sub_assign(&mut self, ticks: u64) {
		self.ticks -= ticks;
	}
}
