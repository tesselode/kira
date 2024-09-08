#[cfg(test)]
mod test;

use std::{sync::Arc, time::Duration};

use crate::sound::{
	EndPosition, IntoOptionalRegion, PlaybackPosition, PlaybackRate, Region, SoundData,
};
use crate::tween::{Tween, Value};
use crate::{Dbfs, StartTime};
use ringbuf::HeapRb;

use super::sound::Shared;
use super::{command_writers_and_readers, StreamingSoundHandle, StreamingSoundSettings};

use super::{
	decoder::Decoder,
	sound::{decode_scheduler::DecodeScheduler, StreamingSound},
};

const ERROR_BUFFER_CAPACITY: usize = 1;

/// A streaming sound that is not playing yet.
pub struct StreamingSoundData<Error: Send + 'static> {
	pub(crate) decoder: Box<dyn Decoder<Error = Error>>,
	/// Settings for the streaming sound.
	pub settings: StreamingSoundSettings,
	/**
	The portion of the sound this [`StreamingSoundData`] represents.

	Note that the [`StreamingSoundData`] holds the entire piece of audio
	it was originally given regardless of the value of `slice`, but
	[`StreamingSoundData::num_frames`] and [`StreamingSoundData::duration`]
	will behave as if this [`StreamingSoundData`] only contained the specified
	portion of audio.
	*/
	pub slice: Option<(usize, usize)>,
}

impl<Error: Send> StreamingSoundData<Error> {
	/// Creates a [`StreamingSoundData`] for a [`Decoder`].
	#[must_use]
	pub fn from_decoder(decoder: impl Decoder<Error = Error> + 'static) -> Self {
		Self {
			decoder: Box::new(decoder),
			settings: StreamingSoundSettings::default(),
			slice: None,
		}
	}

	/**
	Sets when the sound should start playing.

	# Examples

	Configuring a sound to start 4 ticks after a clock's current time:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		sound::streaming::{StreamingSoundData, StreamingSoundSettings},
		clock::ClockSpeed,
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let clock_handle = manager.add_clock(ClockSpeed::TicksPerMinute(120.0))?;
	let sound = StreamingSoundData::from_file("sound.ogg")?
		.start_time(clock_handle.time() + 4);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	#[must_use = "This method consumes self and returns a modified StreamingSoundData, so the return value should be used"]
	pub fn start_time(mut self, start_time: impl Into<StartTime>) -> Self {
		self.settings.start_time = start_time.into();
		self
	}

	/// Sets where in the sound playback should start.
	#[must_use = "This method consumes self and returns a modified StreamingSoundData, so the return value should be used"]
	pub fn start_position(mut self, start_position: impl Into<PlaybackPosition>) -> Self {
		self.settings.start_position = start_position.into();
		self
	}

	/**
	Sets the portion of the sound that should be looped.

	# Examples

	Configure a sound to loop the portion from 3 seconds in to the end:

	```no_run
	# use kira::sound::streaming::StreamingSoundData;
	let sound = StreamingSoundData::from_file("sound.ogg")?.loop_region(3.0..);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Configure a sound to loop the portion from 2 to 4 seconds:

	```no_run
	# use kira::sound::streaming::StreamingSoundData;
	let sound = StreamingSoundData::from_file("sound.ogg")?.loop_region(2.0..4.0);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	#[must_use = "This method consumes self and returns a modified StreamingSoundData, so the return value should be used"]
	pub fn loop_region(mut self, loop_region: impl IntoOptionalRegion) -> Self {
		self.settings.loop_region = loop_region.into_optional_region();
		self
	}

	/**
	Sets the volume of the sound.

	# Examples

	Set the volume as a factor:

	```no_run
	# use kira::sound::streaming::StreamingSoundData;
	let sound = StreamingSoundData::from_file("sound.ogg")?.volume(0.5);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Set the volume as a gain in decibels:

	```no_run
	# use kira::sound::streaming::StreamingSoundData;
	let sound = StreamingSoundData::from_file("sound.ogg")?.volume(kira::Dbfs(-6.0));
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Link the volume to a modulator:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		modulator::tweener::TweenerBuilder,
		sound::streaming::StreamingSoundData,
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.5,
	})?;
	let sound = StreamingSoundData::from_file("sound.ogg")?.volume(&tweener);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	#[must_use = "This method consumes self and returns a modified StreamingSoundData, so the return value should be used"]
	pub fn volume(mut self, volume: impl Into<Value<Dbfs>>) -> Self {
		self.settings.volume = volume.into();
		self
	}

	/**
	Sets the playback rate of the sound.

	Changing the playback rate will change both the speed
	and the pitch of the sound.

	# Examples

	Set the playback rate as a factor:

	```no_run
	# use kira::sound::streaming::StreamingSoundData;
	let sound = StreamingSoundData::from_file("sound.ogg")?.playback_rate(0.5);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Set the playback rate as a change in semitones:

	```no_run
	# use kira::sound::streaming::StreamingSoundData;
	use kira::{Semitones, sound::PlaybackRate};
	let sound = StreamingSoundData::from_file("sound.ogg")?.playback_rate(Semitones(-2.0));
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Link the playback rate to a modulator:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		modulator::tweener::TweenerBuilder,
		sound::streaming::StreamingSoundData,
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.5,
	})?;
	let sound = StreamingSoundData::from_file("sound.ogg")?.playback_rate(&tweener);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	#[must_use = "This method consumes self and returns a modified StreamingSoundData, so the return value should be used"]
	pub fn playback_rate(mut self, playback_rate: impl Into<Value<PlaybackRate>>) -> Self {
		self.settings.playback_rate = playback_rate.into();
		self
	}

	/**
	Sets the panning of the sound, where 0 is hard left
	and 1 is hard right.

	# Examples

	Set the panning to a streaming value:

	``` no_run
	# use kira::sound::streaming::StreamingSoundData;
	let sound = StreamingSoundData::from_file("sound.ogg")?.panning(0.25);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Link the panning to a modulator:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		modulator::tweener::TweenerBuilder,
		sound::streaming::StreamingSoundData,
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.25,
	})?;
	let sound = StreamingSoundData::from_file("sound.ogg")?.panning(&tweener);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	#[must_use = "This method consumes self and returns a modified StreamingSoundData, so the return value should be used"]
	pub fn panning(mut self, panning: impl Into<Value<f64>>) -> Self {
		self.settings.panning = panning.into();
		self
	}

	/// Sets the tween used to fade in the instance from silence.
	#[must_use = "This method consumes self and returns a modified StreamingSoundData, so the return value should be used"]
	pub fn fade_in_tween(mut self, fade_in_tween: impl Into<Option<Tween>>) -> Self {
		self.settings.fade_in_tween = fade_in_tween.into();
		self
	}

	/// Returns the `StreamingSoundData` with the specified settings.
	#[must_use = "This method consumes self and returns a modified StreamingSoundData, so the return value should be used"]
	pub fn with_settings(mut self, settings: StreamingSoundSettings) -> Self {
		self.settings = settings;
		self
	}

	/// Returns the number of frames in the [`StreamingSoundData`].
	///
	/// If [`StreamingSoundData::slice`] is `Some`, this will be the number
	/// of frames in the slice.
	#[must_use]
	pub fn num_frames(&self) -> usize {
		if let Some((start, end)) = self.slice {
			end - start
		} else {
			self.decoder.num_frames()
		}
	}

	/// Returns the duration of the audio.
	///
	/// If [`StreamingSoundData::slice`] is `Some`, this will be the duration
	/// of the slice.
	#[must_use]
	pub fn duration(&self) -> Duration {
		Duration::from_secs_f64(self.num_frames() as f64 / self.decoder.sample_rate() as f64)
	}

	/**
	Sets the portion of the audio this [`StreamingSoundData`] represents.
	*/
	#[must_use = "This method consumes self and returns a modified StreamingSoundData, so the return value should be used"]
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
	) -> Result<StreamingSoundData<crate::sound::FromFileError>, crate::sound::FromFileError> {
		use std::fs::File;

		use super::symphonia::SymphoniaDecoder;

		Ok(Self::from_decoder(SymphoniaDecoder::new(Box::new(
			File::open(path)?,
		))?))
	}

	/// Creates a [`StreamingSoundData`] for a cursor wrapping audio file data.
	pub fn from_cursor<T: AsRef<[u8]> + Send + Sync + 'static>(
		cursor: std::io::Cursor<T>,
	) -> Result<StreamingSoundData<crate::sound::FromFileError>, crate::sound::FromFileError> {
		use super::symphonia::SymphoniaDecoder;

		Ok(Self::from_decoder(SymphoniaDecoder::new(Box::new(cursor))?))
	}

	/// Creates a [`StreamingSoundData`] for a type that implements Symphonia's
	/// [`MediaSource`](symphonia::core::io::MediaSource) trait.
	pub fn from_media_source(
		media_source: impl symphonia::core::io::MediaSource + 'static,
	) -> Result<StreamingSoundData<crate::sound::FromFileError>, crate::sound::FromFileError> {
		use super::symphonia::SymphoniaDecoder;

		Ok(Self::from_decoder(SymphoniaDecoder::new(Box::new(
			media_source,
		))?))
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
