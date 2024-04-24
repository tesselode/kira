use crate::{
	clock::clock_info::ClockInfoProvider,
	modulator::{value_provider::ModulatorValueProvider, Modulator},
};

use super::{ResourceController, SelfReferentialResourceStorage};

pub(crate) struct Modulators(pub(crate) SelfReferentialResourceStorage<Box<dyn Modulator>>);

impl Modulators {
	#[must_use]
	pub fn new(capacity: u16) -> (Self, ResourceController<Box<dyn Modulator>>) {
		let (storage, controller) = SelfReferentialResourceStorage::new(capacity);
		(Self(storage), controller)
	}

	pub fn on_start_processing(&mut self) {
		self.0.remove_and_add(|modulator| modulator.finished());
		for (_, modulator) in &mut self.0 {
			modulator.on_start_processing();
		}
	}

	pub fn process(&mut self, dt: f64, clock_info_provider: &ClockInfoProvider) {
		self.0.for_each(|modulator, others| {
			modulator.update(
				dt,
				clock_info_provider,
				&ModulatorValueProvider::new(others),
			);
		});
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

impl Default for Box<dyn Modulator> {
	fn default() -> Self {
		Box::new(DummyModulator)
	}
}
