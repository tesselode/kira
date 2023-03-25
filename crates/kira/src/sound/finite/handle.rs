use ringbuf::HeapProducer;

use crate::{parameter::Value, tween::Tween, CommandError, PlaybackRate, Volume};

use super::Command;

pub struct SoundHandle {
	pub(super) command_producer: HeapProducer<Command>,
}

impl SoundHandle {
	/// Sets the volume of the sound (as a factor of the original volume).
	pub fn set_volume(
		&mut self,
		volume: impl Into<Value<Volume>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetVolume(volume.into(), tween))
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
		self.command_producer
			.push(Command::SetPlaybackRate(playback_rate.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the panning of the sound, where `0.0` is hard left,
	/// `0.5` is center, and `1.0` is hard right.
	pub fn set_panning(
		&mut self,
		panning: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetPanning(panning.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
