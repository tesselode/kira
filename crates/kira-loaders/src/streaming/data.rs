use kira::sound::SoundData;
use ringbuf::RingBuffer;

use crate::{StreamingSoundHandle, StreamingSoundSettings};

use super::{decoder::Decoder, sound::StreamingSound};

const COMMAND_BUFFER_CAPACITY: usize = 8;
const ERROR_BUFFER_CAPACITY: usize = 8;

/// A streaming sound that is not playing yet.
pub struct StreamingSoundData<Error: Send + 'static> {
	pub(crate) decoder: Box<dyn Decoder<Error = Error>>,
	/// Settings for the streaming sound.
	pub settings: StreamingSoundSettings,
}

impl<Error: Send + 'static> SoundData for StreamingSoundData<Error> {
	type Error = Error;

	type Handle = StreamingSoundHandle<Error>;

	#[allow(clippy::type_complexity)]
	fn into_sound(self) -> Result<(Box<dyn kira::sound::Sound>, Self::Handle), Self::Error> {
		let (command_producer, command_consumer) = RingBuffer::new(COMMAND_BUFFER_CAPACITY).split();
		let (error_producer, error_consumer) = RingBuffer::new(ERROR_BUFFER_CAPACITY).split();
		let sound = StreamingSound::new(self, command_consumer, error_producer)?;
		let shared = sound.shared();
		Ok((
			Box::new(sound),
			StreamingSoundHandle {
				shared,
				command_producer,
				error_consumer,
			},
		))
	}
}
