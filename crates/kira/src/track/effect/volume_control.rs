//! Adjusts the volume of audio.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	clock::clock_info::ClockInfoProvider, command::ValueChangeCommand, command_writers_and_readers,
	dsp::Frame, modulator::value_provider::ModulatorValueProvider, tween::Parameter, Volume,
};

use super::Effect;

struct VolumeControl {
	command_readers: CommandReaders,
	volume: Parameter<Volume>,
}

impl VolumeControl {
	fn new(builder: VolumeControlBuilder, command_readers: CommandReaders) -> Self {
		Self {
			command_readers,
			volume: Parameter::new(builder.0, Volume::Amplitude(1.0)),
		}
	}
}

impl Effect for VolumeControl {
	fn on_start_processing(&mut self) {
		self.volume
			.read_commands(&mut self.command_readers.volume_change);
	}

	fn process(
		&mut self,
		input: Frame,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		self.volume
			.update(dt, clock_info_provider, modulator_value_provider);
		input * self.volume.value().as_amplitude()
	}
}

command_writers_and_readers!(
	struct {
		volume_change: ValueChangeCommand<Volume>
	}
);
