use crate::{command::handle_param_setters, Volume};

use super::{CommandWriters, DistortionKind};

/// Controls a distortion effect.
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
		drive: Volume,

		/// Sets how much dry (unprocessed) signal should be blended
		/// with the wet (processed) signal. `0.0` means only the dry
		/// signal will be heard. `1.0` means only the wet signal will
		/// be heard.
		mix: f64,
	}
}
