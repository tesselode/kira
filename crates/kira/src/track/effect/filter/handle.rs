use crate::{
	command::ValueChangeCommand,
	tween::{Tween, Value},
};

use super::{CommandWriters, FilterMode};

/// Controls a filter effect.
pub struct FilterHandle {
	pub(super) command_writers: CommandWriters,
}

impl FilterHandle {
	/// Sets the frequencies that the filter will remove.
	pub fn set_mode(&mut self, mode: FilterMode) {
		self.command_writers.mode_change.write(mode)
	}

	/// Sets the cutoff frequency of the filter (in hertz).
	pub fn set_cutoff(&mut self, cutoff: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.cutoff_change
			.write(ValueChangeCommand {
				target: cutoff.into(),
				tween,
			})
	}

	/// Sets the resonance of the filter.
	pub fn set_resonance(&mut self, resonance: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.resonance_change
			.write(ValueChangeCommand {
				target: resonance.into(),
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
