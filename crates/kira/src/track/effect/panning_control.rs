//! Adjusts the panning of audio.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	clock::clock_info::ClockInfoProvider, command::ValueChangeCommand, command_writers_and_readers,
	dsp::Frame, modulator::value_provider::ModulatorValueProvider, tween::Parameter,
};

use super::Effect;

struct PanningControl {
	command_readers: CommandReaders,
	panning: Parameter,
}

impl PanningControl {
	fn new(builder: PanningControlBuilder, command_readers: CommandReaders) -> Self {
		Self {
			command_readers,
			panning: Parameter::new(builder.0, 0.5),
		}
	}
}

impl Effect for PanningControl {
	fn on_start_processing(&mut self) {
		self.panning
			.read_commands(&mut self.command_readers.panning_change);
	}

	fn process(
		&mut self,
		input: Frame,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		self.panning
			.update(dt, clock_info_provider, modulator_value_provider);
		input.panned(self.panning.value() as f32)
	}
}

command_writers_and_readers!(
	struct {
		panning_change: ValueChangeCommand<f64>
	}
);
