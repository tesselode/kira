//! Makes a sound harsher and noisier.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use ringbuf::HeapConsumer;

use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	modulator::value_provider::ModulatorValueProvider,
	track::Effect,
	tween::{Parameter, Tween, Value},
	Amplitude, Decibels,
};

enum Command {
	SetKind(DistortionKind),
	SetDrive(Value<Decibels>, Tween),
	SetMix(Value<f64>, Tween),
}

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
	command_consumer: HeapConsumer<Command>,
	kind: DistortionKind,
	drive: Parameter<Decibels>,
	mix: Parameter,
}

impl Effect for Distortion {
	fn on_start_processing(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetKind(kind) => self.kind = kind,
				Command::SetDrive(drive, tween) => self.drive.set(drive, tween),
				Command::SetMix(mix, tween) => self.mix.set(mix, tween),
			}
		}
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
		let drive = Amplitude::from(self.drive.value()).0 as f32;
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
