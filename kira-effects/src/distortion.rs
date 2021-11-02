//! Makes a sound harsher and noisier.

use kira::{
	dsp::Frame,
	manager::resources::Parameters,
	track::Effect,
	value::{cached::CachedValue, Value},
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
}

impl Default for DistortionSettings {
	fn default() -> Self {
		Self {
			kind: Default::default(),
			drive: Value::Fixed(1.0),
		}
	}
}

/// An effect that modifies an input signal to make it more
/// distorted and noisy.
pub struct Distortion {
	kind: DistortionKind,
	drive: CachedValue,
}

impl Distortion {
	/// Creates a new distortion effect.
	pub fn new(settings: DistortionSettings) -> Self {
		Self {
			kind: settings.kind,
			drive: CachedValue::new(.., settings.drive, 1.0),
		}
	}
}

impl Effect for Distortion {
	fn process(&mut self, mut input: Frame, _dt: f64, parameters: &Parameters) -> Frame {
		self.drive.update(parameters);
		let drive = self.drive.get() as f32;
		input *= drive;
		input = match self.kind {
			DistortionKind::HardClip => Frame::new(
				input.left.max(-1.0).min(1.0),
				input.right.max(-1.0).min(1.0),
			),
			DistortionKind::SoftClip => Frame::new(
				input.left / (1.0 + input.left.abs()),
				input.right / (1.0 + input.right.abs()),
			),
		};
		input /= drive;
		input
	}
}
