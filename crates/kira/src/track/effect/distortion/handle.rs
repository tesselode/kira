use crate::{
	command::ValueChangeCommand,
	tween::{Tween, Value},
	Volume,
};

use super::{CommandWriters, DistortionKind};

/// Controls a distortion effect.
pub struct DistortionHandle {
	pub(super) command_writers: CommandWriters,
}

impl DistortionHandle {
	/// Sets the kind of distortion to use.
	pub fn set_kind(&mut self, kind: DistortionKind) {
		self.command_writers.kind_change.write(kind)
	}

	/// Sets how much distortion should be applied.
	pub fn set_drive(&mut self, drive: impl Into<Value<Volume>>, tween: Tween) {
		self.command_writers.drive_change.write(ValueChangeCommand {
			target: drive.into(),
			tween,
		})
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn set_mix(&mut self, mix: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers.mix_change.write(ValueChangeCommand {
			target: mix.into(),
			tween,
		})
	}
}
