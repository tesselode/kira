//! Makes a sound harsher and noisier.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	clock::clock_info::ClockInfoProvider, command::ValueChangeCommand, command_writers_and_readers,
	dsp::Frame, modulator::value_provider::ModulatorValueProvider, track::Effect, tween::Parameter,
	Volume,
};

/// Different types of distortion.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

struct Distortion {
	kind: DistortionKind,
	drive: Parameter<Volume>,
	mix: Parameter,
	command_readers: CommandReaders,
}

impl Effect for Distortion {
	fn on_start_processing(&mut self) {
		if let Some(kind) = self.command_readers.kind_change.read().copied() {
			self.kind = kind;
		}
		self.drive
			.read_commands(&mut self.command_readers.drive_change);
		self.mix.read_commands(&mut self.command_readers.mix_change);
	}

	fn process(
		&mut self,
		input: Frame,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		self.drive
			.update(dt, clock_info_provider, modulator_value_provider);
		self.mix
			.update(dt, clock_info_provider, modulator_value_provider);
		let drive = self.drive.value().as_amplitude() as f32;
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

		let mix = self.mix.value() as f32;
		output * mix.sqrt() + input * (1.0 - mix).sqrt()
	}
}

command_writers_and_readers! {
	kind_change: DistortionKind,
	drive_change: ValueChangeCommand<Volume>,
	mix_change: ValueChangeCommand<f64>
}
