use crate::{
	command::ValueChangeCommand,
	tween::{Tween, Value},
};

use super::CommandWriters;

/// Controls a reverb effect.
pub struct ReverbHandle {
	pub(super) command_writers: CommandWriters,
}

impl ReverbHandle {
	/// Sets how much the room reverberates. A higher value will
	/// result in a bigger sounding room. 1.0 gives an infinitely
	/// reverberating room.
	pub fn set_feedback(&mut self, feedback: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.feedback_change
			.write(ValueChangeCommand {
				target: feedback.into(),
				tween,
			})
	}

	/// Sets how quickly high frequencies disappear from the reverberation.
	pub fn set_damping(&mut self, damping: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.damping_change
			.write(ValueChangeCommand {
				target: damping.into(),
				tween,
			})
	}

	/// Sets the stereo width of the reverb effect (0.0 being fully mono,
	/// 1.0 being fully stereo).
	pub fn set_stereo_width(&mut self, stereo_width: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.stereo_width_change
			.write(ValueChangeCommand {
				target: stereo_width.into(),
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
