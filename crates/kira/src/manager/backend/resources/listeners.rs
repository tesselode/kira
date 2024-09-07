use crate::{info::Info, listener::Listener};

use super::{
	clocks::Clocks, modulators::Modulators, ResourceController, SelfReferentialResourceStorage,
};

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

	pub(crate) fn update(&mut self, dt: f64, clocks: &Clocks, modulators: &Modulators) {
		self.0.for_each(|listener, others| {
			listener.update(
				dt,
				&Info::new(&clocks.0.resources, &modulators.0.resources, others, None),
			);
		});
	}
}
