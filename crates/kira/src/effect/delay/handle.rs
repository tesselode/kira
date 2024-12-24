use crate::{command::handle_param_setters, Decibels, Mix};

use super::CommandWriters;

/// Controls a delay effect.
#[derive(Debug)]
pub struct DelayHandle {
	pub(super) command_writers: CommandWriters,
}

impl DelayHandle {
	handle_param_setters! {
		/// Sets the amount of feedback.
		feedback: Decibels,

		/// Sets how much dry (unprocessed) signal should be blended
		/// with the wet (processed) signal. `0.0` means only the dry
		/// signal will be heard. `1.0` means only the wet signal will
		/// be heard.
		mix: Mix,
	}
}
