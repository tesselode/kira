use crate::{
	effect::{Effect, EffectBuilder},
	Value,
};

use super::{command_writers_and_readers, Doppler, DopplerHandle, DEFAULT_SPEED};

/// Configures a doppler effect.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DopplerBuilder {
	/// The speed of sound in m/s.
	pub speed: Value<f64>,
}

impl DopplerBuilder {
	/// Creates a new [`DopplerBuilder`] with the default settings.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the speed of sound in m/s.
	#[must_use = "This method consumes self and returns a modified DopplerBuilder, so the return value should be used"]
	pub fn speed(self, speed: impl Into<Value<f64>>) -> Self {
		Self {
			speed: speed.into(),
			..self
		}
	}
}

impl Default for DopplerBuilder {
	fn default() -> Self {
		Self {
			speed: Value::Fixed(DEFAULT_SPEED),
		}
	}
}

impl EffectBuilder for DopplerBuilder {
	type Handle = DopplerHandle;

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		(
			Box::new(Doppler::new(self, command_readers)),
			DopplerHandle { command_writers },
		)
	}
}
