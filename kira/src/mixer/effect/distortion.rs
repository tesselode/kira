use crate::{parameter::Parameters, CachedValue, Frame, Value};

use super::Effect;

/// Different types of distortion effect.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize)
)]
pub enum DistortionKind {
	/// The signal will be clamped to the -1.0 to 1.0 range.
	HardClip,
}

impl Default for DistortionKind {
	fn default() -> Self {
		Self::HardClip
	}
}

/// Settings for a [`Distortion`] effect.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct DistortionSettings {
	/// The kind of distortion to use.
	pub kind: DistortionKind,
	/// The factor to multiply the signal by before applying
	/// the distortion.
	pub drive: Value<f64>,
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
	pub fn drive(self, drive: impl Into<Value<f64>>) -> Self {
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
#[derive(Debug, Copy, Clone)]
pub struct Distortion {
	kind: DistortionKind,
	drive: CachedValue<f64>,
}

impl Distortion {
	/// Creates a new distortion effect.
	pub fn new(settings: DistortionSettings) -> Self {
		Self {
			kind: settings.kind,
			drive: CachedValue::new(settings.drive, 1.0),
		}
	}
}

impl Effect for Distortion {
	fn process(&mut self, _dt: f64, mut input: Frame, parameters: &Parameters) -> Frame {
		self.drive.update(parameters);
		let drive = self.drive.value() as f32;
		input *= drive;
		input = match self.kind {
			DistortionKind::HardClip => Frame::new(
				input.left.max(-1.0).min(1.0),
				input.right.max(-1.0).min(1.0),
			),
		};
		input /= drive;
		input
	}
}
