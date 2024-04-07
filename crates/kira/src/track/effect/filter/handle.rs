use crate::handle_param_setters;

use super::{CommandWriters, FilterMode};

/// Controls a filter effect.
pub struct FilterHandle {
	pub(super) command_writers: CommandWriters,
}

impl FilterHandle {
	/// Sets the frequencies that the filter will remove.
	pub fn set_mode(&mut self, mode: FilterMode) {
		self.command_writers.set_mode.write(mode)
	}

	handle_param_setters! {
		/// Sets the cutoff frequency of the filter (in hertz).
		cutoff: f64,

		/// Sets the resonance of the filter.
		resonance: f64,

		/// Sets how much dry (unprocessed) signal should be blended
		/// with the wet (processed) signal. `0.0` means only the dry
		/// signal will be heard. `1.0` means only the wet signal will
		/// be heard.
		mix: f64,
	}
}
