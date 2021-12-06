//! Makes a sound harsher and noisier.

use crate::{dsp::Frame, track::Effect};

/// Different types of distortion effect.
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
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
#[non_exhaustive]
pub struct DistortionSettings {
	/// The kind of distortion to use.
	pub kind: DistortionKind,
	/// The factor to multiply the signal by before applying
	/// the distortion.
	pub drive: f64,
	/// How much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means
	/// only the dry signal will be heard. `1.0` means
	/// only the wet signal will be heard.
	pub mix: f64,
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
	pub fn drive(self, drive: f64) -> Self {
		Self { drive, ..self }
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn mix(self, mix: f64) -> Self {
		Self { mix, ..self }
	}
}

impl Default for DistortionSettings {
	fn default() -> Self {
		Self {
			kind: Default::default(),
			drive: 1.0,
			mix: 1.0,
		}
	}
}

/// An effect that modifies an input signal to make it more
/// distorted and noisy.
pub struct Distortion {
	kind: DistortionKind,
	drive: f64,
	mix: f64,
}

impl Distortion {
	/// Creates a new distortion effect.
	pub fn new(settings: DistortionSettings) -> Self {
		Self {
			kind: settings.kind,
			drive: settings.drive,
			mix: settings.mix,
		}
	}
}

impl Effect for Distortion {
	fn process(&mut self, input: Frame, _dt: f64) -> Frame {
		let drive = self.drive as f32;
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

		let mix = self.mix as f32;
		output * mix.sqrt() + input * (1.0 - mix).sqrt()
	}
}
