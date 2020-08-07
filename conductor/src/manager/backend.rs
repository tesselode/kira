use crate::{project::Project, stereo_sample::StereoSample};
use std::f32::consts::PI;

pub struct Backend {
	dt: f32,
	project: Project,
	phase: f32,
}

impl Backend {
	pub fn new(sample_rate: u32, project: Project) -> Self {
		Self {
			dt: 1.0 / sample_rate as f32,
			project,
			phase: 0.0,
		}
	}

	pub fn process(&mut self) -> StereoSample {
		self.phase += 440.0 * self.dt;
		self.phase %= 1.0;
		StereoSample::from_mono(0.25 * (self.phase * 2.0 * PI).sin())
	}
}
