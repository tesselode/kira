//! Adjusts the volume of audio.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	Decibels, Parameter,
	command::{ValueChangeCommand, read_commands_into_parameters},
	command_writers_and_readers,
	frame::Frame,
	info::Info,
};

use super::Effect;

struct VolumeControl {
	command_readers: CommandReaders,
	volume: Parameter<Decibels>,
}

impl VolumeControl {
	#[must_use]
	fn new(builder: VolumeControlBuilder, command_readers: CommandReaders) -> Self {
		Self {
			command_readers,
			volume: Parameter::new(builder.0, Decibels::IDENTITY),
		}
	}
}

impl Effect for VolumeControl {
	fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, volume);
	}

	fn process(&mut self, input: &mut [Frame], dt: f64, info: &Info) {
		self.volume.update(dt * input.len() as f64, info);
		let num_frames = input.len();
		for (i, frame) in input.iter_mut().enumerate() {
			let time_in_chunk = (i + 1) as f64 / num_frames as f64;
			*frame *= self.volume.interpolated_value(time_in_chunk).as_amplitude();
		}
	}
}

command_writers_and_readers! {
	set_volume: ValueChangeCommand<Decibels>,
}
