//! Adjusts the volume of audio.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	clock::clock_info::ClockInfoProvider, command::read_commands_into_parameters,
	command::ValueChangeCommand, command_writers_and_readers, frame::Frame,
	modulator::value_provider::ModulatorValueProvider, tween::Parameter, Volume,
};

use super::Effect;

struct VolumeControl {
	command_readers: CommandReaders,
	volume: Parameter<Volume>,
}

impl VolumeControl {
	#[must_use]
	fn new(builder: VolumeControlBuilder, command_readers: CommandReaders) -> Self {
		Self {
			command_readers,
			volume: Parameter::new(builder.0, Volume::Amplitude(1.0)),
		}
	}
}

impl Effect for VolumeControl {
	fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, volume);
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
		input * self.volume.value().as_amplitude() as f32
	}
}

command_writers_and_readers! {
	set_volume: ValueChangeCommand<Volume>,
}
