use std::ops::{Deref, DerefMut};

use crate::{
	clock::{Clock, ClockInfo, State},
	INTERNAL_BUFFER_SIZE,
};

pub(crate) struct BufferedClock {
	clock: Clock,
	info_buffer: [ClockInfo; INTERNAL_BUFFER_SIZE],
	current_frame: usize,
}

impl BufferedClock {
	pub fn new(clock: Clock) -> Self {
		Self {
			clock,
			info_buffer: [ClockInfo::default(); INTERNAL_BUFFER_SIZE],
			current_frame: 0,
		}
	}

	pub fn info_buffer(&self) -> [ClockInfo; INTERNAL_BUFFER_SIZE] {
		self.info_buffer
	}

	pub fn update(&mut self, dt: f64) {
		self.clock.update(dt);
		let (ticks, fraction) = match self.clock.state() {
			State::NotStarted => (0, 0.0),
			State::Started {
				ticks,
				fractional_position,
			} => (ticks, fractional_position),
		};
		self.info_buffer[self.current_frame] = ClockInfo {
			ticking: self.clock.ticking(),
			ticks,
			fraction,
		};
		self.current_frame += 1;
	}

	pub fn reset_buffer(&mut self) {
		self.info_buffer = [ClockInfo::default(); INTERNAL_BUFFER_SIZE];
		self.current_frame = 0;
	}
}

impl Default for BufferedClock {
	fn default() -> Self {
		Self {
			clock: Default::default(),
			info_buffer: [ClockInfo::default(); INTERNAL_BUFFER_SIZE],
			current_frame: Default::default(),
		}
	}
}

impl Deref for BufferedClock {
	type Target = Clock;

	fn deref(&self) -> &Self::Target {
		&self.clock
	}
}

impl DerefMut for BufferedClock {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.clock
	}
}
