use kira::sound::{Sound, SoundData};

use crate::StreamingSoundHandle;

use super::{settings::StreamingSoundSettings, sound::StreamingSound, Decoder};

pub struct StreamingSoundData {
	pub decoder: Box<dyn Decoder>,
	pub settings: StreamingSoundSettings,
}

impl StreamingSoundData {
	pub fn new(decoder: impl Decoder + 'static, settings: StreamingSoundSettings) -> Self {
		Self {
			decoder: Box::new(decoder),
			settings,
		}
	}
}

impl SoundData for StreamingSoundData {
	type Handle = StreamingSoundHandle;

	fn into_sound(self) -> (Box<dyn Sound>, Self::Handle) {
		let sound = StreamingSound::new(self);
		let shared = sound.shared();
		(Box::new(sound), StreamingSoundHandle::new(shared))
	}
}
