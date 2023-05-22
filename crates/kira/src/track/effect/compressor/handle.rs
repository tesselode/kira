use std::time::Duration;

use ringbuf::HeapProducer;

use crate::{
	tween::{Tween, Value},
	CommandError,
};

use super::Command;

/// Controls a compressor.
pub struct CompressorHandle {
	pub(super) command_producer: HeapProducer<Command>,
}

impl CompressorHandle {
	/// Sets the volume above which volume will start to be decreased (in dBFS).
	pub fn set_threshold(
		&mut self,
		threshold: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetThreshold(threshold.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets how much the signal will be compressed.
	///
	/// A ratio of `2.0` (or 2 to 1) means an increase of 3dB will
	/// become an increase of 1.5dB. Ratios between `0.0` and `1.0`
	/// will actually expand the audio.
	pub fn set_ratio(
		&mut self,
		ratio: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetRatio(ratio.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets how much time it takes for the volume attenuation to ramp up once
	/// the input volume exceeds the threshold.
	pub fn set_attack_duration(
		&mut self,
		attack_duration: impl Into<Value<Duration>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetAttackDuration(attack_duration.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets how much time it takes for the volume attenuation to relax once
	/// the input volume dips below the threshold.
	pub fn set_release_duration(
		&mut self,
		release_duration: impl Into<Value<Duration>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetReleaseDuration(release_duration.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the amount to change the volume after processing (in dB).
	///
	/// This can be used to compensate for the decrease in volume resulting
	/// from compression. This is only applied to the wet signal, nto the
	/// dry signal.
	pub fn set_makeup_gain(
		&mut self,
		makeup_gain: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetMakeupGain(makeup_gain.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn set_mix(
		&mut self,
		mix: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetMix(mix.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
