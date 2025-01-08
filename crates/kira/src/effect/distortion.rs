//! Makes a sound harsher and noisier.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	command::{read_commands_into_parameters, ValueChangeCommand},
	command_writers_and_readers,
	effect::Effect,
	frame::Frame,
	info::Info,
	Decibels, Mix, Parameter,
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
	drive: Parameter<Decibels>,
	mix: Parameter<Mix>,
}

impl Effect for Distortion {
	fn on_start_processing(&mut self) {
		if let Some(kind) = self.command_readers.set_kind.read() {
			self.kind = kind;
		}
		read_commands_into_parameters!(self, drive, mix);
	}

	fn process(&mut self, input: &mut [Frame], dt: f64, info: &Info) {
		self.drive.update(dt * input.len() as f64, info);
		self.mix.update(dt * input.len() as f64, info);

		let num_frames = input.len();
		for (i, frame) in input.iter_mut().enumerate() {
			let time_in_chunk = (i + 1) as f64 / num_frames as f64;
			let drive = self.drive.interpolated_value(time_in_chunk).as_amplitude();
			let mix = self.mix.interpolated_value(time_in_chunk);

			let mut output = *frame * drive;
			output = match self.kind {
				DistortionKind::HardClip => {
					Frame::new(output.left.clamp(-1.0, 1.0), output.right.clamp(-1.0, 1.0))
				}
				DistortionKind::SoftClip => Frame::new(
					output.left / (1.0 + output.left.abs()),
					output.right / (1.0 + output.right.abs()),
				),
			};
			output /= drive;

			*frame = output * mix.0.sqrt() + *frame * (1.0 - mix.0).sqrt()
		}
	}
}

command_writers_and_readers! {
	set_kind: DistortionKind,
	set_drive: ValueChangeCommand<Decibels>,
	set_mix: ValueChangeCommand<Mix>,
}
