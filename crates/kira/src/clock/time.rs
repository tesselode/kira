#[cfg(test)]
mod test;

use std::{
	cmp::Ordering,
	ops::{Add, AddAssign, Sub, SubAssign},
};

use super::ClockId;

/**
An instant in time associated with a clock.

`ClockTime`s implement [`PartialOrd`]. They can be compared as long
as both times are associated with the same clock. If the clocks are
different, `a.cmp(b)` will return `None`.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClockTime {
	/// The clock this time is associated with.
	pub clock: ClockId,
	/// The elapsed time in whole ticks.
	pub ticks: u64,
	/// The amount of time since the last tick as a fraction of a tick.
	pub fraction: f64,
}

impl ClockTime {
	pub fn from_ticks_u64(clock: impl Into<ClockId>, ticks: u64) -> Self {
		Self {
			clock: clock.into(),
			ticks,
			fraction: 0.0,
		}
	}

	pub fn from_ticks_f64(clock: impl Into<ClockId>, ticks: f64) -> Self {
		Self {
			clock: clock.into(),
			ticks: ticks as u64,
			fraction: ticks.fract(),
		}
	}
}

impl Add<u64> for ClockTime {
	type Output = ClockTime;

	fn add(self, ticks: u64) -> Self::Output {
		Self {
			clock: self.clock,
			ticks: self.ticks + ticks,
			fraction: self.fraction,
		}
	}
}

impl AddAssign<u64> for ClockTime {
	fn add_assign(&mut self, ticks: u64) {
		self.ticks += ticks;
	}
}

impl Add<f64> for ClockTime {
	type Output = ClockTime;

	fn add(self, ticks: f64) -> Self::Output {
		let mut fraction = self.fraction + ticks;
		let mut ticks = self.ticks;
		while fraction >= 1.0 {
			fraction -= 1.0;
			ticks += 1;
		}
		Self {
			clock: self.clock,
			ticks,
			fraction,
		}
	}
}

impl AddAssign<f64> for ClockTime {
	fn add_assign(&mut self, ticks: f64) {
		*self = *self + ticks;
	}
}

impl Sub<u64> for ClockTime {
	type Output = ClockTime;

	fn sub(self, ticks: u64) -> Self::Output {
		Self {
			clock: self.clock,
			ticks: self.ticks - ticks,
			fraction: self.fraction,
		}
	}
}

impl SubAssign<u64> for ClockTime {
	fn sub_assign(&mut self, ticks: u64) {
		self.ticks -= ticks;
	}
}

impl Sub<f64> for ClockTime {
	type Output = ClockTime;

	fn sub(self, ticks: f64) -> Self::Output {
		let mut fraction = self.fraction - ticks;
		let mut ticks = self.ticks;
		while fraction < 0.0 {
			fraction += 1.0;
			ticks -= 1;
		}
		Self {
			clock: self.clock,
			ticks,
			fraction,
		}
	}
}

impl SubAssign<f64> for ClockTime {
	fn sub_assign(&mut self, ticks: f64) {
		*self = *self - ticks;
	}
}

impl PartialOrd for ClockTime {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		if self.clock != other.clock {
			return None;
		}
		match self.ticks.cmp(&other.ticks) {
			Ordering::Equal => self.fraction.partial_cmp(&other.fraction),
			ordering => Some(ordering),
		}
	}
}
