use atomic_arena::{Arena, Controller};
use ringbuf::HeapProducer;

use crate::{
	clock::clock_info::ClockInfoProvider,
	manager::command::ModulatorCommand,
	modulator::{Modulator, ModulatorId},
};

pub(crate) struct Modulators {
	modulators: Arena<Box<dyn Modulator>>,
	unused_modulator_producer: HeapProducer<Box<dyn Modulator>>,
}

impl Modulators {
	pub fn new(
		capacity: usize,
		unused_modulator_producer: HeapProducer<Box<dyn Modulator>>,
	) -> Self {
		Self {
			modulators: Arena::new(capacity),
			unused_modulator_producer,
		}
	}

	pub fn controller(&self) -> Controller {
		self.modulators.controller()
	}

	pub fn get(&self, id: ModulatorId) -> Option<&dyn Modulator> {
		self.modulators
			.get(id.0)
			.map(|modulator| modulator.as_ref())
	}

	pub fn on_start_processing(&mut self) {
		self.remove_unused_modulators();
		for (_, modulator) in &mut self.modulators {
			modulator.on_start_processing();
		}
	}

	pub fn remove_unused_modulators(&mut self) {
		if self.unused_modulator_producer.is_full() {
			return;
		}
		for (_, modulator) in self
			.modulators
			.drain_filter(|modulator| modulator.finished())
		{
			if self.unused_modulator_producer.push(modulator).is_err() {
				panic!("Unused modulator producer is full")
			}
			if self.unused_modulator_producer.is_full() {
				return;
			}
		}
	}

	pub fn run_command(&mut self, command: ModulatorCommand) {
		match command {
			ModulatorCommand::Add(id, modulator) => self
				.modulators
				.insert_with_key(id.0, modulator)
				.expect("Modulator arena is full"),
		}
	}

	pub fn process(&mut self, dt: f64, clock_info_provider: &ClockInfoProvider) {
		for (_, modulator) in &mut self.modulators {
			modulator.update(dt, clock_info_provider);
		}
	}
}
