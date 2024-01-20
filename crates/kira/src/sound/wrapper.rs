use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::{interpolate_frame, Frame},
	modulator::value_provider::ModulatorValueProvider,
	OutputDestination,
};

use super::{CommonSoundSettings, Sound};

pub(crate) struct SoundWrapper {
	pub sound: Box<dyn Sound>,
	pub output_destination: OutputDestination,
	pub time_since_last_frame: f64,
	pub resample_buffer: [Frame; 4],
}

impl SoundWrapper {
	pub fn new(sound: Box<dyn Sound>, settings: CommonSoundSettings) -> Self {
		Self {
			sound,
			output_destination: settings.output_destination,
			time_since_last_frame: 0.0,
			resample_buffer: [Frame::from_mono(0.0); 4],
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
		self.time_since_last_frame += dt;
		while self.time_since_last_frame >= 1.0 / self.sound.sample_rate() {
			self.time_since_last_frame -= 1.0 / self.sound.sample_rate();
			for i in 0..self.resample_buffer.len() - 1 {
				self.resample_buffer[i] = self.resample_buffer[i + 1];
			}
			self.resample_buffer[self.resample_buffer.len() - 1] = self
				.sound
				.process(clock_info_provider, modulator_value_provider);
		}
		interpolate_frame(
			self.resample_buffer[0],
			self.resample_buffer[1],
			self.resample_buffer[2],
			self.resample_buffer[3],
			(self.time_since_last_frame * self.sound.sample_rate()) as f32,
		)
	}
}
