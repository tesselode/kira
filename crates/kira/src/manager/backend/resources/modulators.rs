pub(crate) mod buffered;

use crate::arena::{Arena, Controller};
use ringbuf::HeapProducer;

use crate::{
	clock::clock_info::ClockInfoProvider,
	manager::command::ModulatorCommand,
	modulator::{value_provider::ModulatorValueProvider, Modulator, ModulatorId},
};

use self::buffered::BufferedModulator;

pub(crate) struct Modulators {
	pub(crate) modulators: Arena<BufferedModulator>,
	modulator_ids: Vec<ModulatorId>,
	unused_modulator_producer: HeapProducer<BufferedModulator>,
	dummy_modulator: BufferedModulator,
}

impl Modulators {
	pub fn new(
		capacity: usize,
		unused_modulator_producer: HeapProducer<BufferedModulator>,
	) -> Self {
		Self {
			modulators: Arena::new(capacity),
			modulator_ids: Vec::with_capacity(capacity),
			unused_modulator_producer,
			dummy_modulator: BufferedModulator::new(Box::new(DummyModulator)),
		}
	}

	pub fn controller(&self) -> Controller {
		self.modulators.controller()
	}

	pub fn on_start_processing(&mut self) {
		self.remove_unused_modulators();
		for (_, modulator) in &mut self.modulators {
			modulator.on_start_processing();
		}
	}

	pub fn remove_unused_modulators(&mut self) {
		let mut i = 0;
		while i < self.modulator_ids.len() && !self.unused_modulator_producer.is_full() {
			let id = self.modulator_ids[i];
			let modulator = &mut self.modulators[id.0];
			if modulator.finished() {
				if self
					.unused_modulator_producer
					.push(
						self.modulators
							.remove(id.0)
							.unwrap_or_else(|| panic!("Modulator with ID {:?} does not exist", id)),
					)
					.is_err()
				{
					panic!("Unused modulator producer is full")
				}
				self.modulator_ids.remove(i);
			} else {
				i += 1;
			}
		}
	}

	pub fn run_command(&mut self, command: ModulatorCommand) {
		match command {
			ModulatorCommand::Add(id, modulator) => {
				self.modulators
					.insert_with_key(id.0, modulator)
					.expect("Modulator arena is full");
				self.modulator_ids.push(id);
			}
		}
	}

	pub fn clear_buffers(&mut self) {
		for (_, modulator) in &mut self.modulators {
			modulator.clear_buffer();
		}
	}

	pub fn process(&mut self, dt: f64, clock_info_provider: &ClockInfoProvider) {
		for id in &self.modulator_ids {
			let modulator = self
				.modulators
				.get_mut(id.0)
				.expect("modulator IDs and modulators are out of sync");
			std::mem::swap(modulator, &mut self.dummy_modulator);
			self.dummy_modulator.update(
				dt,
				clock_info_provider,
				&ModulatorValueProvider::latest(&self.modulators),
			);
			let modulator = self
				.modulators
				.get_mut(id.0)
				.expect("modulator IDs and modulators are out of sync");
			std::mem::swap(modulator, &mut self.dummy_modulator);
		}
	}
}

struct DummyModulator;

impl Modulator for DummyModulator {
	fn update(
		&mut self,
		_dt: f64,
		_clock_info_provider: &ClockInfoProvider,
		_modulator_value_provider: &ModulatorValueProvider,
	) {
	}

	fn value(&self) -> f64 {
		0.0
	}

	fn finished(&self) -> bool {
		false
	}
}
