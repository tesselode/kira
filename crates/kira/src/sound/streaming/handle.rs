use std::sync::Arc;

use crate::{
	sound::{IntoOptionalRegion, PlaybackState, Region},
	tween::{Tween, Value},
	CommandError, PlaybackRate, Volume,
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

	/// Sets the volume of the sound (as a factor of the original volume).
	pub fn set_volume(
		&mut self,
		volume: impl Into<Value<Volume>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.sound_command_producer
			.push(SoundCommand::SetVolume(volume.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the playback rate of the sound.
	///
	/// Changing the playback rate will change both the speed
	/// and pitch of the sound.
	pub fn set_playback_rate(
		&mut self,
		playback_rate: impl Into<Value<PlaybackRate>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.sound_command_producer
			.push(SoundCommand::SetPlaybackRate(playback_rate.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the panning of the sound, where `0.0` is hard left,
	/// `0.5` is center, and `1.0` is hard right.
	pub fn set_panning(
		&mut self,
		panning: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.sound_command_producer
			.push(SoundCommand::SetPanning(panning.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the portion of the sound that will be played.
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

	/// Sets the portion of the sound that will play in a loop.
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
