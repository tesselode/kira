use crate::{
	clock::{ClockId, ClockInfo, ClockTime},
	modulator::ModulatorId,
};

use super::{Info, WhenToStart};

pub struct SingleFrameInfo<'a> {
	pub(super) info: &'a Info<'a>,
	pub(super) info_frame: InfoFrame,
}

impl SingleFrameInfo<'_> {
	pub fn clock_info(&self, id: ClockId) -> Option<ClockInfo> {
		match self.info_frame {
			InfoFrame::Latest => self.info.latest_clock_info(id),
			InfoFrame::Specified(frame_index) => self
				.info
				.clock_info(id)
				.and_then(|info| info.get(frame_index))
				.copied(),
		}
	}

	pub fn modulator_value(&self, id: ModulatorId) -> Option<f64> {
		match self.info_frame {
			InfoFrame::Latest => self.info.latest_modulator_value(id),
			InfoFrame::Specified(frame_index) => self
				.info
				.modulator_values(id)
				.and_then(|values| values.get(frame_index))
				.copied(),
		}
	}

	/// Returns whether something with the given start time should
	/// start now, later, or never given the current state of the clocks.
	#[must_use]
	pub fn when_to_start(&self, time: ClockTime) -> WhenToStart {
		if let Some(clock_info) = self.clock_info(time.clock) {
			if clock_info.ticking && clock_info.ticks >= time.ticks {
				WhenToStart::Now
			} else {
				WhenToStart::Later
			}
		} else {
			WhenToStart::Never
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum InfoFrame {
	Latest,
	Specified(usize),
}
