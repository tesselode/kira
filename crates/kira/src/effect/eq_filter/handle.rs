use crate::{Decibels, command::handle_param_setters};

use super::{CommandWriters, EqFilterKind};

/// Controls an EQ filter.
#[derive(Debug)]
pub struct EqFilterHandle {
	pub(super) command_writers: CommandWriters,
}

impl EqFilterHandle {
	/// Sets the shape of the frequency adjustment curve.
	pub fn set_kind(&mut self, kind: EqFilterKind) {
		self.command_writers.set_kind.write(kind)
	}

	handle_param_setters! {
		/// Sets the "center" or "corner" of the frequency range to adjust in Hz
		/// (for bell or shelf curves, respectively).
		frequency: f64,

		/// Sets the volume adjustment for frequencies in the specified range (in decibels).
		gain: Decibels,

		/// Sets the width of the frequency range to adjust.
		///
		/// A higher Q value results in a narrower range of frequencies being adjusted.
		/// The value should be greater than `0.0`.
		q: f64,
	}
}
