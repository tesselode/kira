use crate::sound::{Sound, SoundData};

use super::{sound::StreamingSound, Decoder};

pub struct StreamingSoundData {
	pub decoder: Box<dyn Decoder>,
}

impl StreamingSoundData {
	pub fn new(decoder: impl Decoder + 'static) -> Self {
		Self { decoder: Box::new(decoder) }
	}
}

impl SoundData for StreamingSoundData {
	type Handle = ();

	fn into_sound(self) -> (Box<dyn Sound>, Self::Handle) {
		(Box::new(StreamingSound::new(self)), ())
	}
}
