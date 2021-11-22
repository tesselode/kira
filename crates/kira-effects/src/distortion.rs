//! Makes a sound harsher and noisier.

use kira::{
	dsp::Frame,
	parameter::Parameters,
	track::Effect,
	value::{CachedValue, Value},
};

/// Different types of distortion effect.
#[derive(Debug, Copy, Clone)]
pub enum DistortionKind {
	/// The signal will be clamped to the -1.0 to 1.0 range.
	///
	/// This creates a harsh distortion when the signal leaves
	/// the -1.0 to 1.0 range.
	HardClip,
	/// The signal will be kept in the -1.0 to 1.0 range,
	/// and the slope will gradually decrease as it reaches
	/// -1.0 or 1.0.
	///
	/// This creates a smoother distortion that gradually
	/// becomes more prominent as the signal becomes louder.
	SoftClip,
}

impl Default for DistortionKind {
	fn default() -> Self {
		Self::HardClip
	}
}

/// Settings for a [`Distortion`] effect.
#[derive(Debug, Copy, Clone)]
pub struct DistortionSettings {
	/// The kind of distortion to use.
	pub kind: DistortionKind,
	/// The factor to multiply the signal by before applying
	/// the distortion.
	pub drive: Value,
	/// How much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means
	/// only the dry signal will be heard. `1.0` means
	/// only the wet signal will be heard.
	pub mix: Value,
}

impl DistortionSettings {
	/// Creates a new `DistortionSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the kind of distortion to use.
	pub fn kind(self, kind: DistortionKind) -> Self {
		Self { kind, ..self }
	}

	/// Sets the factor to multiply the signal by before applying
	/// the distortion.
	pub fn drive(self, drive: impl Into<Value>) -> Self {
		Self {
			drive: drive.into(),
			..self
		}
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn mix(self, mix: impl Into<Value>) -> Self {
		Self {
			mix: mix.into(),
			..self
		}
	}
}

impl Default for DistortionSettings {
	fn default() -> Self {
		Self {
			kind: Default::default(),
			drive: Value::Fixed(1.0),
			mix: Value::Fixed(1.0),
		}
	}
}

/// An effect that modifies an input signal to make it more
/// distorted and noisy.
pub struct Distortion {
	kind: DistortionKind,
	drive: CachedValue,
	mix: CachedValue,
}

impl Distortion {
	/// Creates a new distortion effect.
	pub fn new(settings: DistortionSettings) -> Self {
		Self {
			kind: settings.kind,
			drive: CachedValue::new(.., settings.drive, 1.0),
			mix: CachedValue::new(0.0..=1.0, settings.mix, 1.0),
		}
	}
}

impl Effect for Distortion {
	fn process(&mut self, input: Frame, _dt: f64, parameters: &Parameters) -> Frame {
		self.drive.update(parameters);
		let drive = self.drive.get() as f32;
		let mut output = input * drive;
		output = match self.kind {
			DistortionKind::HardClip => Frame::new(
				output.left.max(-1.0).min(1.0),
				output.right.max(-1.0).min(1.0),
			),
			DistortionKind::SoftClip => Frame::new(
				output.left / (1.0 + output.left.abs()),
				output.right / (1.0 + output.right.abs()),
			),
		};
		output /= drive;

		let mix = self.mix.get() as f32;
		output * mix.sqrt() + input * (1.0 - mix).sqrt()
	}
}
