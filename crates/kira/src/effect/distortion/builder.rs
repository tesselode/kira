use crate::{
	Decibels, Mix, Parameter, Value,
	effect::{Effect, EffectBuilder},
};

use super::{Distortion, DistortionKind, command_writers_and_readers, handle::DistortionHandle};

/// Configures a distortion effect.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DistortionBuilder {
	/// The kind of distortion to use.
	pub kind: DistortionKind,
	/// The factor to multiply the signal by before applying
	/// the distortion.
	pub drive: Value<Decibels>,
	/// How much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal.
	pub mix: Value<Mix>,
}

impl DistortionBuilder {
	/// Creates a new [`DistortionBuilder`] with the default settings.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the kind of distortion to use.
	#[must_use = "This method consumes self and returns a modified DistortionBuilder, so the return value should be used"]
	pub fn kind(self, kind: DistortionKind) -> Self {
		Self { kind, ..self }
	}

	/// Sets the factor to multiply the signal by before applying
	/// the distortion.
	#[must_use = "This method consumes self and returns a modified DistortionBuilder, so the return value should be used"]
	pub fn drive(self, drive: impl Into<Value<Decibels>>) -> Self {
		Self {
			drive: drive.into(),
			..self
		}
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	#[must_use = "This method consumes self and returns a modified DistortionBuilder, so the return value should be used"]
	pub fn mix(self, mix: impl Into<Value<Mix>>) -> Self {
		Self {
			mix: mix.into(),
			..self
		}
	}
}

impl Default for DistortionBuilder {
	fn default() -> Self {
		Self {
			kind: Default::default(),
			drive: Value::Fixed(Decibels::IDENTITY),
			mix: Value::Fixed(Mix::WET),
		}
	}
}

impl EffectBuilder for DistortionBuilder {
	type Handle = DistortionHandle;

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		(
			Box::new(Distortion {
				command_readers,
				kind: self.kind,
				drive: Parameter::new(self.drive, Decibels::IDENTITY),
				mix: Parameter::new(self.mix, Mix::WET),
			}),
			DistortionHandle { command_writers },
		)
	}
}
