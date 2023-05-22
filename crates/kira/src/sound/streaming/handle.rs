use std::sync::Arc;

use crate::{
	sound::{IntoOptionalRegion, PlaybackRate, PlaybackState, Region},
	tween::{Tween, Value},
	CommandError, Volume,
};
use ringbuf::{HeapConsumer, HeapProducer};

use super::{sound::Shared, DecodeSchedulerCommand, SoundCommand};

/// Controls a streaming sound.
pub struct StreamingSoundHandle<Error> {
	pub(crate) shared: Arc<Shared>,
	pub(crate) sound_command_producer: HeapProducer<SoundCommand>,
	pub(crate) decode_scheduler_command_producer: HeapProducer<DecodeSchedulerCommand>,
	pub(crate) error_consumer: HeapConsumer<Error>,
}

impl<Error> StreamingSoundHandle<Error> {
	/// Returns the current playback state of the sound.
	pub fn state(&self) -> PlaybackState {
		self.shared.state()
	}

	/// Returns the current playback position of the sound (in seconds).
	pub fn position(&self) -> f64 {
		self.shared.position()
	}

	/**
	Sets the volume of the sound.

	# Examples

	Set the volume of the sound as a factor immediately:

	```no_run
	# use kira::{
	# 	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	# 	sound::streaming::{StreamingSoundData, StreamingSoundSettings},
	# };
	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# let mut sound = manager.play(StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?)?;
	use kira::tween::Tween;

	sound.set_volume(0.5, Tween::default())?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Smoothly transition the volume to a target value in decibels:

	```no_run
	# use kira::{
	# 	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	# 	sound::streaming::{StreamingSoundData, StreamingSoundSettings},
	# };
	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# let mut sound = manager.play(StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?)?;
	use kira::tween::Tween;
	use std::time::Duration;

	sound.set_volume(kira::Volume::Decibels(-6.0), Tween {
		duration: Duration::from_secs(3),
		..Default::default()
	})?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Link the volume to a modulator, smoothly transitioning from the current value:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		sound::streaming::{StreamingSoundData, StreamingSoundSettings},
		modulator::tweener::TweenerBuilder,
		tween::Tween,
	};
	use std::time::Duration;

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.5,
	})?;
	let mut sound = manager.play(StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?)?;
	sound.set_volume(&tweener, Tween {
		duration: Duration::from_secs(3),
		..Default::default()
	})?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn set_volume(
		&mut self,
		volume: impl Into<Value<Volume>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.sound_command_producer
			.push(SoundCommand::SetVolume(volume.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/**
	Sets the playback rate of the sound.

	Changing the playback rate will change both the speed
	and pitch of the sound.

	# Examples

	Set the playback rate of the sound as a factor immediately:

	```no_run
	# use kira::{
	# 	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	# 	sound::streaming::{StreamingSoundData, StreamingSoundSettings},
	# };
	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# let mut sound = manager.play(StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?)?;
	use kira::tween::Tween;

	sound.set_playback_rate(0.5, Tween::default())?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Smoothly transition the playback rate to a target value in semitones:

	```no_run
	# use kira::{
	# 	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	# 	sound::streaming::{StreamingSoundData, StreamingSoundSettings},
	# };
	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# let mut sound = manager.play(StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?)?;
	use kira::{
		tween::Tween,
		sound::PlaybackRate,
	};
	use std::time::Duration;

	sound.set_playback_rate(PlaybackRate::Semitones(-2.0), Tween {
		duration: Duration::from_secs(3),
		..Default::default()
	})?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Link the playback rate to a modulator, smoothly transitioning from the current value:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		sound::streaming::{StreamingSoundData, StreamingSoundSettings},
		modulator::tweener::TweenerBuilder,
		tween::Tween,
	};
	use std::time::Duration;

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.5,
	})?;
	let mut sound = manager.play(StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?)?;
	sound.set_playback_rate(&tweener, Tween {
		duration: Duration::from_secs(3),
		..Default::default()
	})?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn set_playback_rate(
		&mut self,
		playback_rate: impl Into<Value<PlaybackRate>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.sound_command_producer
			.push(SoundCommand::SetPlaybackRate(playback_rate.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/**
	Sets the panning of the sound, where `0.0` is hard left,
	`0.5` is center, and `1.0` is hard right.

	# Examples

	Smoothly transition the panning to a target value:

	```no_run
	# use kira::{
	# 	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	# 	sound::streaming::{StreamingSoundData, StreamingSoundSettings},
	# };
	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# let mut sound = manager.play(StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?)?;
	use kira::tween::Tween;
	use std::time::Duration;

	sound.set_panning(0.25, Tween {
		duration: Duration::from_secs(3),
		..Default::default()
	})?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Link the panning to a modulator, smoothly transitioning from the current value:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		sound::streaming::{StreamingSoundData, StreamingSoundSettings},
		modulator::tweener::TweenerBuilder,
		tween::Tween,
	};
	use std::time::Duration;

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.25,
	})?;
	let mut sound = manager.play(StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?)?;
	sound.set_panning(&tweener, Tween {
		duration: Duration::from_secs(3),
		..Default::default()
	})?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn set_panning(
		&mut self,
		panning: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.sound_command_producer
			.push(SoundCommand::SetPanning(panning.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/**
	Sets the portion of the sound that should be played.

	# Examples

	Set the sound to play from 3 seconds in to the end:

	```no_run
	# use kira::{
	# 	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	# 	sound::streaming::{StreamingSoundData, StreamingSoundSettings},
	# };
	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# let mut sound = manager.play(StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?)?;
	sound.set_playback_region(3.0..)?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Set the sound to play from 2 to 4 seconds:

	```no_run
	# use kira::{
	# 	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	# 	sound::streaming::{StreamingSoundData, StreamingSoundSettings},
	# };
	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# let mut sound = manager.play(StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?)?;
	sound.set_playback_region(2.0..4.0)?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn set_playback_region(
		&mut self,
		playback_region: impl Into<Region>,
	) -> Result<(), CommandError> {
		self.decode_scheduler_command_producer
			.push(DecodeSchedulerCommand::SetPlaybackRegion(
				playback_region.into(),
			))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/**
	Sets the portion of the sound that will play in a loop.

	# Examples

	Set the sound to loop the portion from 3 seconds in to the end:

	```no_run
	# use kira::{
	# 	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	# 	sound::streaming::{StreamingSoundData, StreamingSoundSettings},
	# };
	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# let mut sound = manager.play(StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?)?;
	sound.set_loop_region(3.0..)?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Set the sound to loop the portion from 2 to 4 seconds:

	```no_run
	# use kira::{
	# 	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	# 	sound::streaming::{StreamingSoundData, StreamingSoundSettings},
	# };
	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# let mut sound = manager.play(StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?)?;
	sound.set_loop_region(2.0..4.0)?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Set a sound that was previously looping to stop looping:

	```no_run
	# use kira::{
	# 	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	# 	sound::streaming::{StreamingSoundData, StreamingSoundSettings},
	# };
	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# let mut sound = manager.play(StreamingSoundData::from_file("sound.ogg", StreamingSoundSettings::default())?)?;
	sound.set_loop_region(None)?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn set_loop_region(
		&mut self,
		loop_region: impl IntoOptionalRegion,
	) -> Result<(), CommandError> {
		self.decode_scheduler_command_producer
			.push(DecodeSchedulerCommand::SetLoopRegion(
				loop_region.into_optional_loop_region(),
			))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Fades out the sound to silence with the given tween and then
	/// pauses playback.
	pub fn pause(&mut self, tween: Tween) -> Result<(), CommandError> {
		self.sound_command_producer
			.push(SoundCommand::Pause(tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Resumes playback and fades in the sound from silence
	/// with the given tween.
	pub fn resume(&mut self, tween: Tween) -> Result<(), CommandError> {
		self.sound_command_producer
			.push(SoundCommand::Resume(tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Fades out the sound to silence with the given tween and then
	/// stops playback.
	///
	/// Once the sound is stopped, it cannot be restarted.
	pub fn stop(&mut self, tween: Tween) -> Result<(), CommandError> {
		self.sound_command_producer
			.push(SoundCommand::Stop(tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the playback position to the specified time in seconds.
	pub fn seek_to(&mut self, position: f64) -> Result<(), CommandError> {
		self.decode_scheduler_command_producer
			.push(DecodeSchedulerCommand::SeekTo(position))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Moves the playback position by the specified amount of time in seconds.
	pub fn seek_by(&mut self, amount: f64) -> Result<(), CommandError> {
		self.decode_scheduler_command_producer
			.push(DecodeSchedulerCommand::SeekBy(amount))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Returns an error that occurred while decoding audio, if any.
	pub fn pop_error(&mut self) -> Option<Error> {
		self.error_consumer.pop()
	}
}
