use std::time::Duration;

use ringbuf::HeapRb;

use crate::{
	track::effect::{Effect, EffectBuilder},
	tween::Value,
};

use super::{Compressor, CompressorHandle};

const COMMAND_CAPACITY: usize = 8;

/// Configures a compressor.
pub struct CompressorBuilder {
	/// The volume above which volume will start to be decreased (in dBFS).
	pub threshold: Value<f64>,
	/// How much the signal will be compressed.
	///
	/// A ratio of `2.0` (or 2 to 1) means an increase of 3dB will
	/// become an increase of 1.5dB. Ratios between `0.0` and `1.0`
	/// will actually expand the audio.
	pub ratio: Value<f64>,
	/// How much time it takes for the volume attenuation to ramp up once
	/// the input volume exceeds the threshold.
	pub attack_duration: Value<Duration>,
	/// How much time it takes for the volume attenuation to relax once
	/// the input volume dips below the threshold.
	pub release_duration: Value<Duration>,
	/// The amount to change the volume after processing (in dB).
	///
	/// This can be used to compensate for the decrease in volume resulting
	/// from compression. This is only applied to the wet signal, nto the
	/// dry signal.
	pub makeup_gain: Value<f64>,
	/// How much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means
	/// only the dry signal will be heard. `1.0` means
	/// only the wet signal will be heard.
	pub mix: Value<f64>,
}

impl CompressorBuilder {
	pub(crate) const DEFAULT_THRESHOLD: f64 = 0.0;
	pub(crate) const DEFAULT_RATIO: f64 = 1.0;
	pub(crate) const DEFAULT_ATTACK_DURATION: Duration = Duration::from_millis(10);
	pub(crate) const DEFAULT_RELEASE_DURATION: Duration = Duration::from_millis(100);
	pub(crate) const DEFAULT_MAKEUP_GAIN: f64 = 0.0;
	pub(crate) const DEFAULT_MIX: f64 = 1.0;

	/// Creates a new [`CompressorBuilder`] with the default settings.
	pub fn new() -> Self {
		Self {
			threshold: Value::Fixed(Self::DEFAULT_THRESHOLD),
			ratio: Value::Fixed(Self::DEFAULT_RATIO),
			attack_duration: Value::Fixed(Self::DEFAULT_ATTACK_DURATION),
			release_duration: Value::Fixed(Self::DEFAULT_RELEASE_DURATION),
			makeup_gain: Value::Fixed(Self::DEFAULT_MAKEUP_GAIN),
			mix: Value::Fixed(Self::DEFAULT_MIX),
		}
	}

	/// Sets the volume above which volume will start to be decreased (in dBFS).
	pub fn threshold(self, threshold: impl Into<Value<f64>>) -> Self {
		Self {
			threshold: threshold.into(),
			..self
		}
	}

	/// Sets how much the signal will be compressed.
	///
	/// A ratio of `2.0` (or 2 to 1) means an increase of 3dB will
	/// become an increase of 1.5dB. Ratios between `0.0` and `1.0`
	/// will actually expand the audio.
	pub fn ratio(self, ratio: impl Into<Value<f64>>) -> Self {
		Self {
			ratio: ratio.into(),
			..self
		}
	}

	/// Sets how much time it takes for the volume attenuation to ramp up once
	/// the input volume exceeds the threshold.
	pub fn attack_duration(self, attack_duration: impl Into<Value<Duration>>) -> Self {
		Self {
			attack_duration: attack_duration.into(),
			..self
		}
	}

	/// Sets how much time it takes for the volume attenuation to relax once
	/// the input volume dips below the threshold.
	pub fn release_duration(self, release_duration: impl Into<Value<Duration>>) -> Self {
		Self {
			release_duration: release_duration.into(),
			..self
		}
	}

	/// Sets the amount to change the volume after processing (in dB).
	///
	/// This can be used to compensate for the decrease in volume resulting
	/// from compression. This is only applied to the wet signal, nto the
	/// dry signal.
	pub fn makeup_gain(self, makeup_gain: impl Into<Value<f64>>) -> Self {
		Self {
			makeup_gain: makeup_gain.into(),
			..self
		}
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means
	/// only the dry signal will be heard. `1.0` means
	/// only the wet signal will be heard.
	pub fn mix(self, mix: impl Into<Value<f64>>) -> Self {
		Self {
			mix: mix.into(),
			..self
		}
	}
}

impl Default for CompressorBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl EffectBuilder for CompressorBuilder {
	type Handle = CompressorHandle;

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		let (command_producer, command_consumer) = HeapRb::new(COMMAND_CAPACITY).split();
		(
			Box::new(Compressor::new(self, command_consumer)),
			CompressorHandle { command_producer },
		)
	}
}
