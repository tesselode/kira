use std::ops::{Deref, DerefMut};

use arrayvec::ArrayVec;

use crate::{
	clock::{Clock, ClockInfo},
	info::SingleFrameInfo,
	INTERNAL_BUFFER_SIZE,
};

#[derive(Default)]
pub(crate) struct BufferedClock {
	clock: Clock,
	info_buffer: ArrayVec<ClockInfo, INTERNAL_BUFFER_SIZE>,
}

impl BufferedClock {
	pub fn new(clock: Clock) -> Self {
		Self {
			clock,
			info_buffer: ArrayVec::new(),
		}
	}

	pub fn info_buffer(&self) -> &[ClockInfo] {
		&self.info_buffer
	}

	pub fn update(&mut self, dt: f64, info: &SingleFrameInfo) {
		self.clock.update(dt, info);
		self.info_buffer.push(self.clock.info());
	}

	pub fn clear_buffer(&mut self) {
		self.info_buffer.clear();
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
