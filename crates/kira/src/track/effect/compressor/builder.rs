use std::time::Duration;

use ringbuf::HeapRb;

use crate::{
	track::effect::{Effect, EffectBuilder},
	tween::Value,
};

use super::{Compressor, CompressorHandle};

const COMMAND_CAPACITY: usize = 8;

pub struct CompressorBuilder {
	pub threshold: Value<f64>,
	pub ratio: Value<f64>,
	pub attack_duration: Value<Duration>,
	pub release_duration: Value<Duration>,
	pub makeup_gain: Value<f64>,
	pub mix: Value<f64>,
}

impl CompressorBuilder {
	pub const DEFAULT_THRESHOLD: f64 = 0.0;
	pub const DEFAULT_RATIO: f64 = 1.0;
	pub const DEFAULT_ATTACK_DURATION: Duration = Duration::from_millis(10);
	pub const DEFAULT_RELEASE_DURATION: Duration = Duration::from_millis(100);
	pub const DEFAULT_MAKEUP_GAIN: f64 = 0.0;
	pub const DEFAULT_MIX: f64 = 1.0;

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

	pub fn threshold(self, threshold: impl Into<Value<f64>>) -> Self {
		Self {
			threshold: threshold.into(),
			..self
		}
	}

	pub fn ratio(self, ratio: impl Into<Value<f64>>) -> Self {
		Self {
			ratio: ratio.into(),
			..self
		}
	}

	pub fn attack_duration(self, attack_duration: impl Into<Value<Duration>>) -> Self {
		Self {
			attack_duration: attack_duration.into(),
			..self
		}
	}

	pub fn release_duration(self, release_duration: impl Into<Value<Duration>>) -> Self {
		Self {
			release_duration: release_duration.into(),
			..self
		}
	}

	pub fn makeup_gain(self, makeup_gain: impl Into<Value<f64>>) -> Self {
		Self {
			makeup_gain: makeup_gain.into(),
			..self
		}
	}

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
