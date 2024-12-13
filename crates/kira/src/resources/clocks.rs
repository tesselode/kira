pub(crate) mod buffered_clock;

use buffered_clock::BufferedClock;

use super::{ResourceController, SelfReferentialResourceStorage};

pub(crate) struct Clocks(pub(crate) SelfReferentialResourceStorage<BufferedClock>);

impl Clocks {
	#[must_use]
	pub(crate) fn new(capacity: u16) -> (Self, ResourceController<BufferedClock>) {
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

	pub(crate) fn reset_buffers(&mut self) {
		for (_, clock) in &mut self.0 {
			clock.reset_buffer();
		}
	}

	pub(crate) fn update(&mut self, dt: f64) {
		self.0.for_each(|clock, others| {
			clock.update(dt);
		});
	}
}
