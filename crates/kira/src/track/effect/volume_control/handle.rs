use crate::{
	command::ValueChangeCommand,
	tween::{Tween, Value},
	Volume,
};

use super::CommandWriters;

/// Controls a volume control effect.
pub struct VolumeControlHandle {
	pub(super) command_writers: CommandWriters,
}

impl VolumeControlHandle {
	/// Sets the volume adjustment to apply to input audio.
	pub fn set_volume(&mut self, volume: impl Into<Value<Volume>>, tween: Tween) {
		self.command_writers
			.volume_change
			.write(ValueChangeCommand {
				target: volume.into(),
				tween,
			})
	}
}
