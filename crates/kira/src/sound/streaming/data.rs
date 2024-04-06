#[cfg(test)]
mod test;

use std::{sync::Arc, time::Duration};

use crate::sound::{EndPosition, IntoOptionalRegion, Region, SoundData};
use ringbuf::HeapRb;

use super::sound::Shared;
use super::{command_writers_and_readers, StreamingSoundHandle, StreamingSoundSettings};

use super::{
	decoder::Decoder,
	sound::{decode_scheduler::DecodeScheduler, StreamingSound},
};

const ERROR_BUFFER_CAPACITY: usize = 8;

/// A streaming sound that is not playing yet.
pub struct StreamingSoundData<Error: Send + 'static> {
	pub(crate) decoder: Box<dyn Decoder<Error = Error>>,
	/// Settings for the streaming sound.
	pub settings: StreamingSoundSettings,
	pub slice: Option<(usize, usize)>,
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
			slice: None,
		}
	}

	pub fn num_frames(&self) -> usize {
		if let Some((start, end)) = self.slice {
			end - start
		} else {
			self.decoder.num_frames()
		}
	}

	/// Returns the duration of the audio.
	pub fn duration(&self) -> Duration {
		Duration::from_secs_f64(self.num_frames() as f64 / self.decoder.sample_rate() as f64)
	}

	pub fn slice(mut self, region: impl IntoOptionalRegion) -> Self {
		self.slice = region.into_optional_region().map(|Region { start, end }| {
			let start = start.into_samples(self.decoder.sample_rate());
			let end = match end {
				EndPosition::EndOfAudio => self.decoder.num_frames(),
				EndPosition::Custom(end) => end.into_samples(self.decoder.sample_rate()),
			};
			(start, end)
		});
		self
	}
}

impl<T: Send> StreamingSoundData<T> {}

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
		let (command_writers, command_readers, decode_scheduler_command_readers) =
			command_writers_and_readers();
		let (error_producer, error_consumer) = HeapRb::new(ERROR_BUFFER_CAPACITY).split();
		let sample_rate = self.decoder.sample_rate();
		let shared = Arc::new(Shared::new());
		let (scheduler, frame_consumer) = DecodeScheduler::new(
			self.decoder,
			self.slice,
			self.settings,
			shared.clone(),
			decode_scheduler_command_readers,
			error_producer,
		)?;
		let sound = StreamingSound::new(
			sample_rate,
			self.settings,
			shared.clone(),
			frame_consumer,
			command_readers,
			&scheduler,
		);
		let handle = StreamingSoundHandle {
			shared,
			command_writers,
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
