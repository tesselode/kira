use crate::{info::Info, modulator::Modulator};

use super::{
	ResourceController, SelfReferentialResourceStorage, clocks::Clocks, listeners::Listeners,
};

pub(crate) struct Modulators(pub(crate) SelfReferentialResourceStorage<Box<dyn Modulator>>);

impl Modulators {
	#[must_use]
	pub fn new(capacity: usize) -> (Self, ResourceController<Box<dyn Modulator>>) {
		let (storage, controller) = SelfReferentialResourceStorage::new(capacity);
		(Self(storage), controller)
	}

	pub fn on_start_processing(&mut self) {
		self.0.remove_and_add(|modulator| modulator.finished());
		for (_, modulator) in &mut self.0 {
			modulator.on_start_processing();
		}
	}

	pub fn process(&mut self, dt: f64, clocks: &Clocks, listeners: &Listeners) {
		self.0.for_each(|modulator, others| {
			modulator.update(
				dt,
				&Info::new(&clocks.0.resources, others, &listeners.0.resources, None),
			);
		});
	}
}

struct DummyModulator;

impl Modulator for DummyModulator {
	fn update(&mut self, _dt: f64, _info: &Info) {}

	fn value(&self) -> f64 {
		0.0
	}

	fn finished(&self) -> bool {
		false
	}
}

impl Default for Box<dyn Modulator> {
	fn default() -> Self {
		Box::new(DummyModulator)
	}
}
