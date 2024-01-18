use crate::{
	clock::clock_info::ClockInfoProvider, dsp::Frame,
	modulator::value_provider::ModulatorValueProvider, OutputDestination,
};

use super::{CommonSoundSettings, Sound};

pub(crate) struct SoundWrapper {
	pub sound: Box<dyn Sound>,
	pub output_destination: OutputDestination,
}

impl SoundWrapper {
	pub fn new(sound: Box<dyn Sound>, settings: CommonSoundSettings) -> Self {
		Self {
			sound,
			output_destination: settings.output_destination,
		}
	}

	pub fn output_destination(&self) -> OutputDestination {
		self.output_destination
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		todo!()
	}
}
