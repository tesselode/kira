use crate::{
	clock::{clock_info::ClockInfoProvider, Clock},
	modulator::value_provider::ModulatorValueProvider,
};

use super::{ResourceController, SelfReferentialResourceStorage};

pub(crate) struct Clocks(pub(crate) SelfReferentialResourceStorage<Clock>);

impl Clocks {
	pub(crate) fn new(capacity: u16) -> (Self, ResourceController<Clock>) {
		let (storage, controller) = SelfReferentialResourceStorage::new(capacity);
		(Self(storage), controller)
	}

	pub(crate) fn on_start_processing(&mut self) {
		self.0
			.remove_and_add(|clock| clock.shared().is_marked_for_removal());
		for (_, clock) in &mut self.0 {
			clock.on_start_processing();
		}
	}

	pub(crate) fn update(&mut self, dt: f64, modulator_value_provider: &ModulatorValueProvider) {
		self.0.for_each(|clock, others| {
			clock.update(
				dt,
				&ClockInfoProvider::new(others),
				modulator_value_provider,
			);
		});
	}
}
