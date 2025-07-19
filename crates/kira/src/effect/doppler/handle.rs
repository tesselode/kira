use crate::command::handle_param_setters;

use super::CommandWriters;

/// Controls a reverb effect.
#[derive(Debug)]
pub struct DopplerHandle {
	pub(super) command_writers: CommandWriters,
}

impl DopplerHandle {
	handle_param_setters! {
		/// Sets the speed of sound in m/s.
		speed: f64,
	}
}
