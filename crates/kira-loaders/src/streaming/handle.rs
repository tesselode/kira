use std::{fmt::Display, sync::Arc};

use kira::{sound::static_sound::PlaybackState, tween::Tween, value::Value};
use ringbuf::{Consumer, Producer};

use crate::Error;

use super::{sound::Shared, Command};

/// An error that occurs when trying to modify a streaming sound
/// whose command queue is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandQueueFull;

impl Display for CommandQueueFull {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("Cannot send a command to the sound because the command queue is full")
	}
}

impl std::error::Error for CommandQueueFull {}

/// Controls a streaming sound.
pub struct StreamingSoundHandle {
	pub(crate) shared: Arc<Shared>,
	pub(crate) command_producer: Producer<Command>,
	pub(crate) error_consumer: Consumer<Error>,
}

impl StreamingSoundHandle {
	/// Returns the current playback state of the sound.
	pub fn state(&self) -> PlaybackState {
		self.shared.state()
	}

	/// Returns the current playback position of the sound (in seconds).
	pub fn position(&self) -> f64 {
		self.shared.position()
	}

	/// Sets the volume of the sound (as a factor of the original volume).
	pub fn set_volume(&mut self, volume: impl Into<Value>) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SetVolume(volume.into()))
			.map_err(|_| CommandQueueFull)
	}

	/// Sets the playback rate of the sound (as a factor of the
	/// original speed).
	///
	/// Changing the playback rate will change both the speed
	/// and pitch of the sound.
	pub fn set_playback_rate(
		&mut self,
		playback_rate: impl Into<Value>,
	) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SetPlaybackRate(playback_rate.into()))
			.map_err(|_| CommandQueueFull)
	}

	/// Sets the panning of the sound, where `0.0` is hard left,
	/// `0.5` is center, and `1.0` is hard right.
	pub fn set_panning(&mut self, panning: impl Into<Value>) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SetPanning(panning.into()))
			.map_err(|_| CommandQueueFull)
	}

	/// Fades out the sound to silence with the given tween and then
	/// pauses playback.
	pub fn pause(&mut self, tween: Tween) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::Pause(tween))
			.map_err(|_| CommandQueueFull)
	}

	/// Resumes playback and fades in the sound from silence
	/// with the given tween.
	pub fn resume(&mut self, tween: Tween) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::Resume(tween))
			.map_err(|_| CommandQueueFull)
	}

	/// Fades out the sound to silence with the given tween and then
	/// stops playback.
	///
	/// Once the sound is stopped, it cannot be restarted.
	pub fn stop(&mut self, tween: Tween) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::Stop(tween))
			.map_err(|_| CommandQueueFull)
	}

	/// Sets the playback position to the specified time in seconds.
	pub fn seek_to(&mut self, position: f64) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SeekTo(position))
			.map_err(|_| CommandQueueFull)
	}

	/// Moves the playback position by the specified amount of time in seconds.
	pub fn seek_by(&mut self, amount: f64) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SeekBy(amount))
			.map_err(|_| CommandQueueFull)
	}

	/// Returns an error that occurred while decoding audio, if any.
	pub fn pop_error(&mut self) -> Option<Error> {
		self.error_consumer.pop()
	}
}
