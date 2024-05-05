//! Makes a sound harsher and noisier.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	clock::clock_info::ClockInfoProvider, command::read_commands_into_parameters,
	command::ValueChangeCommand, command_writers_and_readers, effect::Effect, frame::Frame,
	modulator::value_provider::ModulatorValueProvider, tween::Parameter, Volume,
};

/// Different types of distortion.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
	command_readers: CommandReaders,
	kind: DistortionKind,
	drive: Parameter<Volume>,
	mix: Parameter,
}

impl Effect for Distortion {
	fn on_start_processing(&mut self) {
		if let Some(kind) = self.command_readers.set_kind.read() {
			self.kind = kind;
		}
		read_commands_into_parameters!(self, drive, mix);
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
	set_kind: DistortionKind,
	set_drive: ValueChangeCommand<Volume>,
	set_mix: ValueChangeCommand<f64>,
}
