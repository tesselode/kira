//! Adjusts the volume of audio.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	command::{read_commands_into_parameters, ValueChangeCommand},
	command_writers_and_readers,
	frame::Frame,
	info::Info,
	tween::Parameter,
	Decibels, INTERNAL_BUFFER_SIZE,
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
		let mut volume_buffer = [Decibels::IDENTITY; INTERNAL_BUFFER_SIZE];
		self.volume
			.update_chunk(&mut volume_buffer[..input.len()], dt, info);
		for (frame, volume) in input.iter_mut().zip(volume_buffer.iter().copied()) {
			*frame *= volume.as_amplitude();
		}
	}
}

command_writers_and_readers! {
	set_volume: ValueChangeCommand<Decibels>,
}
