use std::time::Duration;

use crate::{
	command::ValueChangeCommand,
	tween::{Tween, Value},
};

use super::CommandWriters;

/// Controls a compressor.
pub struct CompressorHandle {
	pub(super) command_writers: CommandWriters,
}

impl CompressorHandle {
	/// Sets the volume above which volume will start to be decreased (in dBFS).
	pub fn set_threshold(&mut self, threshold: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.threshold_change
			.write(ValueChangeCommand {
				target: threshold.into(),
				tween,
			})
	}

	/// Sets how much the signal will be compressed.
	///
	/// A ratio of `2.0` (or 2 to 1) means an increase of 3dB will
	/// become an increase of 1.5dB. Ratios between `0.0` and `1.0`
	/// will actually expand the audio.
	pub fn set_ratio(&mut self, ratio: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers.ratio_change.write(ValueChangeCommand {
			target: ratio.into(),
			tween,
		})
	}

	/// Sets how much time it takes for the volume attenuation to ramp up once
	/// the input volume exceeds the threshold.
	pub fn set_attack_duration(
		&mut self,
		attack_duration: impl Into<Value<Duration>>,
		tween: Tween,
	) {
		self.command_writers
			.attack_duration_change
			.write(ValueChangeCommand {
				target: attack_duration.into(),
				tween,
			})
	}

	/// Sets how much time it takes for the volume attenuation to relax once
	/// the input volume dips below the threshold.
	pub fn set_release_duration(
		&mut self,
		release_duration: impl Into<Value<Duration>>,
		tween: Tween,
	) {
		self.command_writers
			.release_duration_change
			.write(ValueChangeCommand {
				target: release_duration.into(),
				tween,
			})
	}

	/// Sets the amount to change the volume after processing (in dB).
	///
	/// This can be used to compensate for the decrease in volume resulting
	/// from compression. This is only applied to the wet signal, nto the
	/// dry signal.
	pub fn set_makeup_gain(&mut self, makeup_gain: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.makeup_gain_change
			.write(ValueChangeCommand {
				target: makeup_gain.into(),
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
