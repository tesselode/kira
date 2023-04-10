use std::time::Duration;

use ringbuf::HeapRb;

use crate::{
	parameter::Value,
	track::effect::{Effect, EffectBuilder},
};

use super::{Compressor, CompressorHandle};

const COMMAND_CAPACITY: usize = 8;

pub struct CompressorBuilder {
	pub threshold: Value<f32>,
	pub ratio: Value<f32>,
	pub attack_duration: Value<Duration>,
	pub release_duration: Value<Duration>,
}

impl CompressorBuilder {
	pub const DEFAULT_THRESHOLD: f32 = -24.0;
	pub const DEFAULT_RATIO: f32 = 8.0;
	pub const DEFAULT_ATTACK_DURATION: Duration = Duration::from_millis(10);
	pub const DEFAULT_RELEASE_DURATION: Duration = Duration::from_millis(100);

	pub fn new() -> Self {
		Self {
			threshold: Value::Fixed(Self::DEFAULT_THRESHOLD),
			ratio: Value::Fixed(Self::DEFAULT_RATIO),
			attack_duration: Value::Fixed(Self::DEFAULT_ATTACK_DURATION),
			release_duration: Value::Fixed(Self::DEFAULT_RELEASE_DURATION),
		}
	}

	pub fn threshold(self, threshold: impl Into<Value<f32>>) -> Self {
		Self {
			threshold: threshold.into(),
			..self
		}
	}

	pub fn ratio(self, ratio: impl Into<Value<f32>>) -> Self {
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
