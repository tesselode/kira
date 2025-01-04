use crate::{command::handle_param_setters, Decibels, Mix};

use super::{CommandWriters, DistortionKind};

/// Controls a distortion effect.
#[derive(Debug)]
pub struct DistortionHandle {
	pub(super) command_writers: CommandWriters,
}

impl DistortionHandle {
	/// Sets the kind of distortion to use.
	pub fn set_kind(&mut self, kind: DistortionKind) {
		self.command_writers.set_kind.write(kind)
	}

	handle_param_setters! {
		/// Sets how much distortion should be applied.
		drive: Decibels,

		/// Sets how much dry (unprocessed) signal should be blended
		/// with the wet (processed) signal.
		mix: Mix,
	}
}
