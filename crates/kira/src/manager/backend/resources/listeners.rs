use crate::{
	clock::clock_info::ClockInfoProvider,
	listener::{Listener, ListenerInfoProvider},
	modulator::value_provider::ModulatorValueProvider,
};

use super::{ResourceController, SelfReferentialResourceStorage};

pub(crate) struct Listeners(pub(crate) SelfReferentialResourceStorage<Listener>);

impl Listeners {
	#[must_use]
	pub(crate) fn new(capacity: u16) -> (Self, ResourceController<Listener>) {
		let (storage, controller) = SelfReferentialResourceStorage::new(capacity);
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
		self.0.for_each(|listener, others| {
			listener.update(
				dt,
				clock_info_provider,
				modulator_value_provider,
				&ListenerInfoProvider::new(None, others),
			);
		});
	}
}
