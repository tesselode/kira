//! Adjusts the panning of audio.

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
	Panning,
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

	fn process(&mut self, input: Frame, dt: f64, info: &Info) -> Frame {
		self.panning.update(dt, info);
		input.panned(self.panning.value())
	}
}

command_writers_and_readers! {
	set_panning: ValueChangeCommand<Panning>,
}
