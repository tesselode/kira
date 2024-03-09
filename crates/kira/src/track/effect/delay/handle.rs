use crate::{
	command::ValueChangeCommand,
	tween::{Tween, Value},
	Volume,
};

use super::CommandWriters;

/// Controls a delay effect.
pub struct DelayHandle {
	pub(super) command_writers: CommandWriters,
}

impl DelayHandle {
	/// Sets the delay time (in seconds).
	pub fn set_delay_time(&mut self, delay_time: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.delay_time_change
			.write(ValueChangeCommand {
				target: delay_time.into(),
				tween,
			})
	}

	/// Sets the amount of feedback.
	pub fn set_feedback(&mut self, feedback: impl Into<Value<Volume>>, tween: Tween) {
		self.command_writers
			.feedback_change
			.write(ValueChangeCommand {
				target: feedback.into(),
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
