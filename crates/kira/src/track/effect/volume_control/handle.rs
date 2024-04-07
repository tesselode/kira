use crate::{handle_param_setters, Volume};

use super::CommandWriters;

/// Controls a volume control effect.
pub struct VolumeControlHandle {
	pub(super) command_writers: CommandWriters,
}

impl VolumeControlHandle {
	handle_param_setters! {
		/// Sets the volume adjustment to apply to input audio.
		volume: Volume,
	}
}
