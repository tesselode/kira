use ringbuf::HeapProducer;

use crate::{
	tween::{Tween, Value},
	CommandError, Volume,
};

use super::Command;

/// Controls a volume control effect.
pub struct VolumeControlHandle {
	pub(super) command_producer: HeapProducer<Command>,
}

impl VolumeControlHandle {
	/// Sets the volume adjustment to apply to input audio.
	pub fn set_volume(
		&mut self,
		volume: impl Into<Value<Volume>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetVolume(volume.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
