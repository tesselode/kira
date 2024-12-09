use crate::{sound::Sound, Frame, INTERNAL_BUFFER_SIZE};

use super::{ResourceController, ResourceStorage};

pub(crate) struct Sounds(pub(crate) ResourceStorage<Box<dyn Sound>>);

impl Sounds {
	#[must_use]
	pub(crate) fn new(capacity: u16) -> (Self, ResourceController<Box<dyn Sound>>) {
		let (storage, controller) = ResourceStorage::new(capacity);
		(Self(storage), controller)
	}

	pub(crate) fn on_start_processing(&mut self) {
		self.0.remove_and_add(|sound| sound.finished());
		for (_, sound) in &mut self.0 {
			sound.on_start_processing();
		}
	}

	pub(crate) fn process(&mut self, dt: f64) -> [Frame; INTERNAL_BUFFER_SIZE] {
		let mut frames = [Frame::ZERO; INTERNAL_BUFFER_SIZE];
		for (_, sound) in &mut self.0 {
			let sound_out = sound.process();
			for (i, frame) in sound_out.iter().copied().enumerate() {
				frames[i] += frame;
			}
		}
		frames
	}
}
