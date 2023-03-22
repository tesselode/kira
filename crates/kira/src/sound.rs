/*!
Sources of audio.
*/

mod handle;

pub use handle::*;

use crate::{
	clock::clock_info::ClockInfoProvider, dsp::Frame,
	modulator::value_provider::ModulatorValueProvider, OutputDestination,
};

pub struct Sound;

impl Sound {
	pub fn output_destination(&mut self) -> OutputDestination {
		todo!()
	}

	pub(crate) fn on_start_processing(&mut self) {
		todo!()
	}

	pub(crate) fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		todo!()
	}

	pub(crate) fn finished(&self) -> bool {
		todo!()
	}
}
