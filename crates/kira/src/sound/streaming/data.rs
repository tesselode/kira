use std::fs::File;
use std::io::Cursor;
use std::path::Path;

use crate::sound::FromFileError;
use crate::sound::SoundData;
use ringbuf::RingBuffer;

use super::decoder::symphonia::SymphoniaDecoder;
use super::{StreamingSoundHandle, StreamingSoundSettings};

use super::{
	decoder::Decoder,
	sound::{decode_scheduler::DecodeScheduler, StreamingSound},
};

const COMMAND_BUFFER_CAPACITY: usize = 8;
const ERROR_BUFFER_CAPACITY: usize = 8;

/// A streaming sound that is not playing yet.
pub struct StreamingSoundData<Error: Send + 'static = FromFileError> {
	pub(crate) decoder: Box<dyn Decoder<Error = Error>>,
	/// Settings for the streaming sound.
	pub settings: StreamingSoundSettings,
}

impl StreamingSoundData<FromFileError> {
	/// Creates a [`StreamingSoundData`] for an audio file.
	pub fn stream(
		path: impl AsRef<Path>,
		settings: StreamingSoundSettings,
	) -> Result<StreamingSoundData<FromFileError>, FromFileError> {
		Ok(StreamingSoundData {
			decoder: Box::new(SymphoniaDecoder::new(Box::new(File::open(path)?))?),
			settings,
		})
	}

	/// Creates a [`StreamingSoundData`] for a cursor wrapping audio file data.
	pub fn stream_from_cursor<T: AsRef<[u8]> + Send + Sync + 'static>(
		cursor: Cursor<T>,
		settings: StreamingSoundSettings,
	) -> Result<StreamingSoundData<FromFileError>, FromFileError> {
		Ok(StreamingSoundData {
			decoder: Box::new(SymphoniaDecoder::new(Box::new(cursor))?),
			settings,
		})
	}
}

impl<Error: Send + 'static> StreamingSoundData<Error> {
	pub(crate) fn split(
		self,
	) -> Result<
		(
			StreamingSound,
			StreamingSoundHandle<Error>,
			DecodeScheduler<Error>,
		),
		Error,
	> {
		let (command_producer, command_consumer) = RingBuffer::new(COMMAND_BUFFER_CAPACITY).split();
		let (error_producer, error_consumer) = RingBuffer::new(ERROR_BUFFER_CAPACITY).split();
		let sample_rate = self.decoder.sample_rate();
		let (scheduler, scheduler_controller) =
			DecodeScheduler::new(self.decoder, self.settings, error_producer)?;
		let sound = StreamingSound::new(
			command_consumer,
			scheduler_controller,
			self.settings,
			sample_rate,
			&scheduler,
		);
		let handle = StreamingSoundHandle {
			shared: sound.shared(),
			command_producer,
			error_consumer,
		};
		Ok((sound, handle, scheduler))
	}
}

impl<Error: Send + 'static> SoundData for StreamingSoundData<Error> {
	type Error = Error;

	type Handle = StreamingSoundHandle<Error>;

	#[allow(clippy::type_complexity)]
	fn into_sound(self) -> Result<(Box<dyn crate::sound::Sound>, Self::Handle), Self::Error> {
		let (sound, handle, scheduler) = self.split()?;
		scheduler.start();
		Ok((Box::new(sound), handle))
	}
}
