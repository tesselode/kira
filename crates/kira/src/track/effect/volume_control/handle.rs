use ringbuf::Producer;

use crate::{tween::Tween, CommandError, Volume};

use super::Command;

/// Controls a volume control effect.
pub struct VolumeControlHandle {
	pub(super) command_producer: Producer<Command>,
}

impl VolumeControlHandle {
	/// Sets the volume adjustment to apply to input audio.
	pub fn set_volume(
		&mut self,
		volume: impl Into<Volume>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetVolume(volume.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
