use std::sync::Arc;

use crate::{
	clock::{
		clock_info::{ClockInfo, ClockInfoProvider},
		Clock, ClockShared,
	},
	manager::backend::Renderer,
	modulator::value_provider::ModulatorValueProvider,
};

pub(crate) struct BufferedClock {
	clock: Clock,
	info_buffer: Vec<ClockInfo>,
}

impl BufferedClock {
	pub(crate) fn new(clock: Clock) -> Self {
		Self {
			clock,
			info_buffer: Vec::with_capacity(Renderer::INTERNAL_BUFFER_SIZE),
		}
	}

	pub(crate) fn shared(&self) -> Arc<ClockShared> {
		self.clock.shared()
	}

	pub(crate) fn on_start_processing(&mut self) {
		self.clock.on_start_processing()
	}

	pub(crate) fn clear_buffer(&mut self) {
		self.info_buffer.clear();
	}

	pub(crate) fn update(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Option<u64> {
		let new_tick = self
			.clock
			.update(dt, clock_info_provider, modulator_value_provider);
		self.info_buffer.push(ClockInfo {
			ticking: self.clock.ticking(),
			ticks: self.clock.ticks(),
			fractional_position: self.clock.fractional_position(),
		});
		new_tick
	}

	pub(crate) fn latest_info(&self) -> ClockInfo {
		ClockInfo {
			ticking: self.clock.ticking(),
			ticks: self.clock.ticks(),
			fractional_position: self.clock.fractional_position(),
		}
	}

	pub(crate) fn info_at_index(&self, index: usize) -> ClockInfo {
		self.info_buffer[index]
	}
}
