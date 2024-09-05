use crate::{
	clock::clock_info::ClockInfoProvider, listener::Listener,
	modulator::value_provider::ModulatorValueProvider,
};

use super::{ResourceController, ResourceStorage};

pub(crate) struct Listeners(pub(crate) ResourceStorage<Listener>);

impl Listeners {
	#[must_use]
	pub(crate) fn new(capacity: u16) -> (Self, ResourceController<Listener>) {
		let (storage, controller) = ResourceStorage::new(capacity);
		(Self(storage), controller)
	}

	pub(crate) fn on_start_processing(&mut self) {
		self.0
			.remove_and_add(|listener| listener.shared.is_marked_for_removal());
		for (_, listener) in &mut self.0 {
			listener.on_start_processing();
		}
	}

	pub(crate) fn update(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) {
		for (_, listener) in &mut self.0 {
			listener.update(dt, clock_info_provider, modulator_value_provider);
		}
	}
}
