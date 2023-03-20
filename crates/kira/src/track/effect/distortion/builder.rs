use ringbuf::HeapRb;

use crate::{
	parameter::{Parameter, Value},
	track::effect::{Effect, EffectBuilder},
	Volume,
};

use super::{handle::DistortionHandle, Distortion, DistortionKind};

const COMMAND_CAPACITY: usize = 8;

/// Configures a distortion effect.
#[derive(Debug, Copy, Clone, PartialEq)]
#[non_exhaustive]
pub struct DistortionBuilder {
	/// The kind of distortion to use.
	pub kind: DistortionKind,
	/// The factor to multiply the signal by before applying
	/// the distortion.
	pub drive: Value<Volume>,
	/// How much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means
	/// only the dry signal will be heard. `1.0` means
	/// only the wet signal will be heard.
	pub mix: Value<f64>,
}

impl DistortionBuilder {
	/// Creates a new [`DistortionBuilder`] with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the kind of distortion to use.
	pub fn kind(self, kind: DistortionKind) -> Self {
		Self { kind, ..self }
	}

	/// Sets the factor to multiply the signal by before applying
	/// the distortion.
	pub fn drive(self, drive: impl Into<Value<Volume>>) -> Self {
		Self {
			drive: drive.into(),
			..self
		}
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn mix(self, mix: impl Into<Value<f64>>) -> Self {
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
			drive: Value::Fixed(Volume::Amplitude(1.0)),
			mix: Value::Fixed(1.0),
		}
	}
}

impl EffectBuilder for DistortionBuilder {
	type Handle = DistortionHandle;

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		let (command_producer, command_consumer) = HeapRb::new(COMMAND_CAPACITY).split();
		(
			Box::new(Distortion {
				command_consumer,
				kind: self.kind,
				drive: Parameter::new(self.drive, Volume::Amplitude(1.0)),
				mix: Parameter::new(self.mix, 1.0),
			}),
			DistortionHandle { command_producer },
		)
	}
}
