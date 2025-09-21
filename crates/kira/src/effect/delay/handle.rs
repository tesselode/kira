use crate::{Decibels, Mix, command::handle_param_setters};

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
		/// with the wet (processed) signal.
		mix: Mix,
	}
}
