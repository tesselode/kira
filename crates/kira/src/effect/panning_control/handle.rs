use crate::command::handle_param_setters;

use super::CommandWriters;

/// Controls a panning control effect.
pub struct PanningControlHandle {
	pub(super) command_writers: CommandWriters,
}

impl PanningControlHandle {
	handle_param_setters! {
		/// Sets the panning adjustment to apply to input audio.
		panning: f64,
	}
}
