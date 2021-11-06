use kira::sound::{Sound, SoundData};

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
	type Handle = ();

	fn into_sound(self) -> (Box<dyn Sound>, Self::Handle) {
		(Box::new(StreamingSound::new(self)), ())
	}
}
