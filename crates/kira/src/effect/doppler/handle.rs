use crate::command::handle_param_setters;

use super::CommandWriters;

/// Controls a reverb effect.
#[derive(Debug)]
pub struct DopplerHandle {
	pub(super) command_writers: CommandWriters,
}

impl DopplerHandle {
	handle_param_setters! {
		/// Sets the temperature in celsius.
		temperature: f64,

		/// Sets the molar mass in kg/mol.
		mass: f64,
	}
}
