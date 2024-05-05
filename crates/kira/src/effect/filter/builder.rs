use crate::{
	effect::{Effect, EffectBuilder},
	tween::Value,
};

use super::{command_writers_and_readers, Filter, FilterHandle, FilterMode};

/// Configures a filter effect.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FilterBuilder {
	/// The frequencies that the filter will remove.
	pub mode: FilterMode,
	/// The cutoff frequency of the filter (in hertz).
	pub cutoff: Value<f64>,
	/// The resonance of the filter.
	///
	/// The resonance is a feedback effect that produces
	/// a distinctive "ringing" sound.
	pub resonance: Value<f64>,
	/// How much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means
	/// only the dry signal will be heard. `1.0` means
	/// only the wet signal will be heard.
	pub mix: Value<f64>,
}

impl FilterBuilder {
	/// Creates a new [`FilterBuilder`] with the default settings.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the frequencies that the filter will remove.
	#[must_use = "This method consumes self and returns a modified FilterBuilder, so the return value should be used"]
	pub fn mode(self, mode: FilterMode) -> Self {
		Self { mode, ..self }
	}

	/// Sets the cutoff frequency of the filter (in hertz).
	#[must_use = "This method consumes self and returns a modified FilterBuilder, so the return value should be used"]
	pub fn cutoff(self, cutoff: impl Into<Value<f64>>) -> Self {
		Self {
			cutoff: cutoff.into(),
			..self
		}
	}

	/// Sets the resonance of the filter.
	#[must_use = "This method consumes self and returns a modified FilterBuilder, so the return value should be used"]
	pub fn resonance(self, resonance: impl Into<Value<f64>>) -> Self {
		Self {
			resonance: resonance.into(),
			..self
		}
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	#[must_use = "This method consumes self and returns a modified FilterBuilder, so the return value should be used"]
	pub fn mix(self, mix: impl Into<Value<f64>>) -> Self {
		Self {
			mix: mix.into(),
			..self
		}
	}
}

impl Default for FilterBuilder {
	fn default() -> Self {
		Self {
			mode: FilterMode::LowPass,
			cutoff: Value::Fixed(1000.0),
			resonance: Value::Fixed(0.0),
			mix: Value::Fixed(1.0),
		}
	}
}

impl EffectBuilder for FilterBuilder {
	type Handle = FilterHandle;

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		(
			Box::new(Filter::new(self, command_readers)),
			FilterHandle { command_writers },
		)
	}
}
