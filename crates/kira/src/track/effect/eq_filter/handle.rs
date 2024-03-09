use crate::{
	command::ValueChangeCommand,
	tween::{Tween, Value},
};

use super::{CommandWriters, EqFilterKind};

/// Controls an EQ filter.
pub struct EqFilterHandle {
	pub(super) command_writers: CommandWriters,
}

impl EqFilterHandle {
	/// Sets the shape of the frequency adjustment curve.
	pub fn set_kind(&mut self, kind: EqFilterKind) {
		self.command_writers.kind_change.write(kind)
	}

	/// Sets the "center" or "corner" of the frequency range to adjust in Hz
	/// (for bell or shelf curves, respectively).
	pub fn set_frequency(&mut self, frequency: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.frequency_change
			.write(ValueChangeCommand {
				target: frequency.into(),
				tween,
			})
	}

	/// Sets the volume adjustment for frequencies in the specified range (in decibels).
	pub fn set_gain(&mut self, gain: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers.gain_change.write(ValueChangeCommand {
			target: gain.into(),
			tween,
		})
	}

	/// Sets the width of the frequency range to adjust.
	///
	/// A higher Q value results in a narrower range of frequencies being adjusted.
	/// The value should be greater than `0.0`.
	pub fn set_q(&mut self, q: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers.q_change.write(ValueChangeCommand {
			target: q.into(),
			tween,
		})
	}
}
