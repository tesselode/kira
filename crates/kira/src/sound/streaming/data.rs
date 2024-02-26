#[cfg(test)]
mod test;

use std::{sync::Arc, time::Duration};

use crate::sound::{CommonSoundController, CommonSoundSettings, EndPosition, Region, SoundData};
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
}

impl<T: Send> StreamingSoundData<T> {
	pub fn num_frames(&self) -> usize {
		self.slice
			.map(|(start, end)| end - start)
			.unwrap_or(self.decoder.num_frames())
	}

	/// Returns the duration of the audio.
	pub fn duration(&self) -> Duration {
		Duration::from_secs_f64(self.num_frames() as f64 / self.decoder.sample_rate() as f64)
	}

	pub fn sliced(self, region: impl Into<Region>) -> Self {
		let Region { start, end } = region.into();
		let slice = (
			start.into_samples(self.decoder.sample_rate()),
			match end {
				EndPosition::EndOfAudio => self.decoder.num_frames(),
				EndPosition::Custom(end) => end.into_samples(self.decoder.sample_rate()),
			},
		);
		Self {
			slice: Some(slice),
			..self
		}
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
		common_controller: CommonSoundController,
	) -> Result<
		(
			StreamingSound,
			StreamingSoundHandle<Error>,
			DecodeScheduler<Error>,
		),
		Error,
	> {
		let (command_writers, sound_command_readers, decode_scheduler_command_readers) =
			command_writers_and_readers();
		let (error_producer, error_consumer) = HeapRb::new(ERROR_BUFFER_CAPACITY).split();
		let sample_rate = self.decoder.sample_rate();
		let shared = Arc::new(Shared::new());
		let (scheduler, frame_consumer) = DecodeScheduler::new(
			self.decoder,
			self.slice,
			self.settings,
			shared.clone(),
			error_producer,
			decode_scheduler_command_readers,
		)?;
		let sound = StreamingSound::new(
			sample_rate,
			self.settings,
			shared.clone(),
			frame_consumer,
			&scheduler,
			sound_command_readers,
		);
		let handle = StreamingSoundHandle {
			shared,
			common_controller,
			error_consumer,
			command_writers,
		};
		Ok((sound, handle, scheduler))
	}

	#[cfg(test)]
	pub(crate) fn split_without_handle(
		self,
	) -> Result<(StreamingSound, DecodeScheduler<Error>), Error> {
		let (_, sound_command_readers, decode_scheduler_command_readers) =
			command_writers_and_readers();
		let (error_producer, _) = HeapRb::new(ERROR_BUFFER_CAPACITY).split();
		let sample_rate = self.decoder.sample_rate();
		let shared = Arc::new(Shared::new());
		let (scheduler, frame_consumer) = DecodeScheduler::new(
			self.decoder,
			self.slice,
			self.settings,
			shared.clone(),
			error_producer,
			decode_scheduler_command_readers,
		)?;
		let sound = StreamingSound::new(
			sample_rate,
			self.settings,
			shared.clone(),
			frame_consumer,
			&scheduler,
			sound_command_readers,
		);
		Ok((sound, scheduler))
	}
}

impl<Error: Send + 'static> SoundData for StreamingSoundData<Error> {
	type Error = Error;

	type Handle = StreamingSoundHandle<Error>;

	fn common_settings(&self) -> CommonSoundSettings {
		CommonSoundSettings {
			start_time: self.settings.start_time,
			volume: self.settings.volume,
			panning: self.settings.panning,
			output_destination: self.settings.output_destination,
			fade_in_tween: self.settings.fade_in_tween,
		}
	}

	#[allow(clippy::type_complexity)]
	fn into_sound(
		self,
		common_controller: CommonSoundController,
	) -> Result<(Box<dyn crate::sound::Sound>, Self::Handle), Self::Error> {
		let (sound, handle, scheduler) = self.split(common_controller)?;
		scheduler.start();
		Ok((Box::new(sound), handle))
	}
}
