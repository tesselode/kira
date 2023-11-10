use std::sync::Arc;
use std::time::Duration;

use crate::sound::SoundData;
use ringbuf::HeapRb;

use super::sound::Shared;
use super::{StreamingSoundHandle, StreamingSoundSettings};

use super::{
	decoder::Decoder,
	sound::{decode_scheduler::DecodeScheduler, StreamingSound},
};

const COMMAND_BUFFER_CAPACITY: usize = 8;
const ERROR_BUFFER_CAPACITY: usize = 8;

/// A streaming sound that is not playing yet.
pub struct StreamingSoundData<Error: Send + 'static> {
	pub(crate) decoder: Box<dyn Decoder<Error = Error>>,
	/// Settings for the streaming sound.
	pub settings: StreamingSoundSettings,
}

impl<Error: Send> StreamingSoundData<Error> {
	/// Creates a [`StreamingSoundData`] for a [`Decoder`].
	pub fn from_decoder(
		decoder: impl Decoder<Error = Error> + 'static,
		settings: StreamingSoundSettings,
	) -> Self {
		Self {
			decoder: Box::new(decoder),
			settings,
		}
	}

	/// Returns the duration of the audio.
	pub fn duration(&self) -> Option<Duration> {
		self.decoder.num_frames().map(|num_frames| {
			Duration::from_secs_f64(num_frames as f64 / self.decoder.sample_rate() as f64)
		})
	}
}

#[cfg(feature = "symphonia")]
impl StreamingSoundData<crate::sound::FromFileError> {
	/// Creates a [`StreamingSoundData`] for an audio file.
	pub fn from_file(
		path: impl AsRef<std::path::Path>,
		settings: StreamingSoundSettings,
	) -> Result<StreamingSoundData<crate::sound::FromFileError>, crate::sound::FromFileError> {
		use std::fs::File;

		use super::symphonia::SymphoniaDecoder;

		Ok(Self::from_decoder(
			SymphoniaDecoder::new(Box::new(File::open(path)?))?,
			settings,
		))
	}

	/// Creates a [`StreamingSoundData`] for a cursor wrapping audio file data.
	pub fn from_cursor<T: AsRef<[u8]> + Send + Sync + 'static>(
		cursor: std::io::Cursor<T>,
		settings: StreamingSoundSettings,
	) -> Result<StreamingSoundData<crate::sound::FromFileError>, crate::sound::FromFileError> {
		use super::symphonia::SymphoniaDecoder;

		Ok(Self::from_decoder(
			SymphoniaDecoder::new(Box::new(cursor))?,
			settings,
		))
	}

	/// Creates a [`StreamingSoundData`] for a type that implements Symphonia's
	/// [`MediaSource`](symphonia::core::io::MediaSource) trait.
	pub fn from_media_source(
		media_source: impl symphonia::core::io::MediaSource + 'static,
		settings: StreamingSoundSettings,
	) -> Result<StreamingSoundData<crate::sound::FromFileError>, crate::sound::FromFileError> {
		use super::symphonia::SymphoniaDecoder;

		Ok(Self::from_decoder(
			SymphoniaDecoder::new(Box::new(media_source))?,
			settings,
		))
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
		let (sound_command_producer, sound_command_consumer) =
			HeapRb::new(COMMAND_BUFFER_CAPACITY).split();
		let (decode_scheduler_command_producer, decode_scheduler_command_consumer) =
			HeapRb::new(COMMAND_BUFFER_CAPACITY).split();
		let (error_producer, error_consumer) = HeapRb::new(ERROR_BUFFER_CAPACITY).split();
		let sample_rate = self.decoder.sample_rate();
		let shared = Arc::new(Shared::new());
		let (scheduler, frame_consumer) = DecodeScheduler::new(
			self.decoder,
			self.settings,
			shared.clone(),
			decode_scheduler_command_consumer,
			error_producer,
		)?;
		let sound = StreamingSound::new(
			sample_rate,
			self.settings,
			shared.clone(),
			frame_consumer,
			sound_command_consumer,
			&scheduler,
		);
		let handle = StreamingSoundHandle {
			shared,
			sound_command_producer,
			decode_scheduler_command_producer,
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
