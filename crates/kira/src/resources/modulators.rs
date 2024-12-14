pub(crate) mod buffered_modulator;

use buffered_modulator::BufferedModulator;

use crate::{arena::Arena, info::Info};

use super::{BufferedClock, ResourceController, SelfReferentialResourceStorage};

pub(crate) struct Modulators(pub(crate) SelfReferentialResourceStorage<BufferedModulator>);

impl Modulators {
	#[must_use]
	pub(crate) fn new(capacity: u16) -> (Self, ResourceController<BufferedModulator>) {
		let (storage, controller) = SelfReferentialResourceStorage::new(capacity);
		(Self(storage), controller)
	}

	pub(crate) fn on_start_processing(&mut self) {
		self.0.remove_and_add(|modulator| modulator.finished());
		for (_, modulator) in &mut self.0 {
			modulator.on_start_processing();
		}
	}

	pub(crate) fn reset_buffers(&mut self) {
		for (_, modulator) in &mut self.0 {
			modulator.clear_buffer();
		}
	}

	pub(crate) fn update(&mut self, dt: f64, clocks: &Arena<BufferedClock>) {
		self.0.for_each(|modulator, others| {
			let info = Info::new(clocks, others);
			let single_frame_info = info.latest();
			modulator.update(dt, &single_frame_info);
		});
	}
}
