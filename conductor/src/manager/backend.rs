use crate::{project::Project, stereo_sample::StereoSample};
use ringbuf::Consumer;
use std::f32::consts::PI;

pub enum Command {
	Test,
}

pub struct Backend {
	dt: f32,
	project: Project,
	command_consumer: Consumer<Command>,
	phase: f32,
	volume: f32,
}

impl Backend {
	pub fn new(sample_rate: u32, project: Project, command_consumer: Consumer<Command>) -> Self {
		Self {
			dt: 1.0 / sample_rate as f32,
			project,
			command_consumer,
			phase: 0.0,
			volume: 0.0,
		}
	}

	pub fn process_commands(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::Test => {
					self.volume = 0.25;
				}
			}
		}
	}

	pub fn process(&mut self) -> StereoSample {
		self.process_commands();
		self.phase += 440.0 * self.dt;
		self.phase %= 1.0;
		StereoSample::from_mono(self.volume * (self.phase * 2.0 * PI).sin())
	}
}
