use ringbuf::RingBuffer;

use crate::{
	track::effect::{Effect, EffectBuilder},
	tween::Tweener,
	Volume,
};

use super::{handle::DistortionHandle, Distortion, DistortionKind};

const COMMAND_CAPACITY: usize = 8;

/// Configures a distortion effect.
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub struct DistortionBuilder {
	/// The kind of distortion to use.
	pub kind: DistortionKind,
	/// The factor to multiply the signal by before applying
	/// the distortion.
	pub drive: Volume,
	/// How much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means
	/// only the dry signal will be heard. `1.0` means
	/// only the wet signal will be heard.
	pub mix: f64,
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
	pub fn drive(self, drive: impl Into<Volume>) -> Self {
		Self {
			drive: drive.into(),
			..self
		}
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn mix(self, mix: f64) -> Self {
		Self { mix, ..self }
	}
}

impl Default for DistortionBuilder {
	fn default() -> Self {
		Self {
			kind: Default::default(),
			drive: Volume::Amplitude(1.0),
			mix: 1.0,
		}
	}
}

impl EffectBuilder for DistortionBuilder {
	type Handle = DistortionHandle;

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		let (command_producer, command_consumer) = RingBuffer::new(COMMAND_CAPACITY).split();
		(
			Box::new(Distortion {
				command_consumer,
				kind: self.kind,
				drive: Tweener::new(self.drive),
				mix: Tweener::new(self.mix),
			}),
			DistortionHandle { command_producer },
		)
	}
}
