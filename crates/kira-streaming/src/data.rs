use kira::sound::{Sound, SoundData};
use ringbuf::RingBuffer;

use crate::StreamingSoundHandle;

use super::{settings::StreamingSoundSettings, sound::StreamingSound, Decoder};

const COMMAND_BUFFER_CAPACITY: usize = 8;

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
		let (command_producer, command_consumer) = RingBuffer::new(COMMAND_BUFFER_CAPACITY).split();
		let sound = StreamingSound::new(self, command_consumer);
		let shared = sound.shared();
		(
			Box::new(sound),
			StreamingSoundHandle {
				shared,
				command_producer,
			},
		)
	}
}
