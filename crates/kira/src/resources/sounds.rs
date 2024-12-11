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
		self.0.remove_and_add(|clock| clock.finished());
		for (_, clock) in &mut self.0 {
			clock.on_start_processing();
		}
	}

	pub(crate) fn process(&mut self, out: &mut [Frame], dt: f64) {
		let mut per_sound_buffer = [Frame::ZERO; INTERNAL_BUFFER_SIZE];
		for (_, sound) in &mut self.0 {
			sound.process(&mut per_sound_buffer[..out.len()], dt);
			for (summed_out, sound_out) in out.iter_mut().zip(per_sound_buffer.iter_mut()) {
				*summed_out += *sound_out;
			}
			per_sound_buffer = [Frame::ZERO; INTERNAL_BUFFER_SIZE];
		}
	}
}
