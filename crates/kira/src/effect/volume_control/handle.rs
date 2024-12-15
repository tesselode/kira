use crate::{command::handle_param_setters, Decibels};

use super::CommandWriters;

/// Controls a volume control effect.
#[derive(Debug)]
pub struct VolumeControlHandle {
	pub(super) command_writers: CommandWriters,
}

impl VolumeControlHandle {
	handle_param_setters! {
		/// Sets the volume adjustment to apply to input audio.
		volume: Decibels,
	}
}
