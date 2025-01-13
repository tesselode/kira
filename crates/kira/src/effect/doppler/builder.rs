use crate::{
	effect::{Effect, EffectBuilder},
	Value,
};

use super::{command_writers_and_readers, Doppler, DopplerHandle};

/// Configures a doppler effect.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DopplerBuilder {
	/// The temperature in celsius.
	pub temperature: Value<f64>,
	/// The molar mass in kg/mol.
	pub mass: Value<f64>,
	/// The adiabetic index.
	pub index: Value<f64>,
}

impl DopplerBuilder {
	/// Creates a new [`DopplerBuilder`] with the default settings.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the temperature.
	#[must_use = "This method consumes self and returns a modified DopplerBuilder, so the return value should be used"]
	pub fn temperature(self, temperature: impl Into<Value<f64>>) -> Self {
		Self {
			temperature: temperature.into(),
			..self
		}
	}

	/// Sets the molar mass in kg/mol.
	#[must_use = "This method consumes self and returns a modified DopplerBuilder, so the return value should be used"]
	pub fn mass(self, mass: impl Into<Value<f64>>) -> Self {
		Self {
			mass: mass.into(),
			..self
		}
	}

	/// Sets the adiabetic index.
	#[must_use = "This method consumes self and returns a modified DopplerBuilder, so the return value should be used"]
	pub fn index(self, index: impl Into<Value<f64>>) -> Self {
		Self {
			index: index.into(),
			..self
		}
	}
}

impl Default for DopplerBuilder {
	fn default() -> Self {
		Self {
			temperature: Value::Fixed(20.0), // Celsius
			mass: Value::Fixed(0.02897),     // air
			index: Value::Fixed(1.4),        // air
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
