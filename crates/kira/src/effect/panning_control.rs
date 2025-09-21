//! Adjusts the panning of audio.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	Panning, Parameter,
	command::{ValueChangeCommand, read_commands_into_parameters},
	command_writers_and_readers,
	frame::Frame,
	info::Info,
};

use super::Effect;

struct PanningControl {
	command_readers: CommandReaders,
	panning: Parameter<Panning>,
}

impl PanningControl {
	#[must_use]
	fn new(builder: PanningControlBuilder, command_readers: CommandReaders) -> Self {
		Self {
			command_readers,
			panning: Parameter::new(builder.0, Panning::CENTER),
		}
	}
}

impl Effect for PanningControl {
	fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, panning);
	}

	fn process(&mut self, input: &mut [Frame], dt: f64, info: &Info) {
		self.panning.update(dt * input.len() as f64, info);
		let num_frames = input.len();
		for (i, frame) in input.iter_mut().enumerate() {
			let time_in_chunk = (i + 1) as f64 / num_frames as f64;
			*frame = frame.panned(self.panning.interpolated_value(time_in_chunk))
		}
	}
}

command_writers_and_readers! {
	set_panning: ValueChangeCommand<Panning>,
}
