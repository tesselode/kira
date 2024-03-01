use crate::arena::{Arena, Controller};
use ringbuf::HeapProducer;

use crate::{
	clock::{self, clock_info::ClockInfoProvider, Clock, ClockId, ClockSpeed},
	modulator::value_provider::ModulatorValueProvider,
};

pub(crate) struct Clocks {
	pub(crate) clocks: Arena<Clock>,
	clock_ids: Vec<ClockId>,
	unused_clock_producer: HeapProducer<Clock>,
	dummy_clock: Clock,
}

impl Clocks {
	pub(crate) fn new(capacity: u16, unused_clock_producer: HeapProducer<Clock>) -> Self {
		Self {
			clocks: Arena::new(capacity),
			clock_ids: Vec::with_capacity(capacity as usize),
			unused_clock_producer,
			dummy_clock: {
				let (_, command_readers) = clock::command_writers_and_readers();
				Clock::new(ClockSpeed::SecondsPerTick(1.0).into(), command_readers)
			},
		}
	}

	pub(crate) fn controller(&self) -> Controller {
		self.clocks.controller()
	}

	pub(crate) fn on_start_processing(&mut self) {
		self.remove_unused_clocks();
		for (_, clock) in &mut self.clocks {
			clock.on_start_processing();
		}
	}

	fn remove_unused_clocks(&mut self) {
		let mut i = 0;
		while i < self.clock_ids.len() && !self.unused_clock_producer.is_full() {
			let id = self.clock_ids[i];
			let clock = &mut self.clocks[id.0];
			if clock.shared().is_marked_for_removal() {
				if self
					.unused_clock_producer
					.push(
						self.clocks
							.remove(id.0)
							.unwrap_or_else(|| panic!("Clock with ID {:?} does not exist", id)),
					)
					.is_err()
				{
					panic!("Unused clock producer is full")
				}
				self.clock_ids.remove(i);
			} else {
				i += 1;
			}
		}
	}

	pub(crate) fn add_clock(&mut self, id: ClockId, clock: Clock) {
		self.clocks
			.insert_with_key(id.0, clock)
			.expect("Clock arena is full");
		self.clock_ids.push(id);
	}

	pub(crate) fn update(&mut self, dt: f64, modulator_value_provider: &ModulatorValueProvider) {
		for id in &self.clock_ids {
			std::mem::swap(
				&mut self.dummy_clock,
				self.clocks
					.get_mut(id.0)
					.expect("clock IDs and clocks are out of sync"),
			);
			self.dummy_clock.update(
				dt,
				&ClockInfoProvider::new(&self.clocks),
				modulator_value_provider,
			);
			std::mem::swap(
				&mut self.dummy_clock,
				self.clocks
					.get_mut(id.0)
					.expect("clock IDs and clocks are out of sync"),
			);
		}
	}
}
