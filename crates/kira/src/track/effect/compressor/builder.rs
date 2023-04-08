use crate::{
	parameter::Value,
	track::effect::{Effect, EffectBuilder},
};

use super::Compressor;

pub struct CompressorBuilder {
	pub threshold: Value<f32>,
	pub ratio: Value<f32>,
	pub attack_speed: Value<f32>,
	pub release_speed: Value<f32>,
}

impl CompressorBuilder {
	pub fn new() -> Self {
		Self {
			threshold: Value::Fixed(-12.0),
			ratio: Value::Fixed(2.0),
			attack_speed: Value::Fixed(1.0),
			release_speed: Value::Fixed(1.0),
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

	pub fn attack_speed(self, attack_speed: impl Into<Value<f32>>) -> Self {
		Self {
			attack_speed: attack_speed.into(),
			..self
		}
	}

	pub fn release_speed(self, release_speed: impl Into<Value<f32>>) -> Self {
		Self {
			release_speed: release_speed.into(),
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
	type Handle = ();

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		(Box::new(Compressor::new(self)), ())
	}
}
