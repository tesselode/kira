use crate::{project::Project, stereo_sample::StereoSample};

pub struct Backend {
	sample_rate: u32,
	project: Project,
}

impl Backend {
	pub fn new(sample_rate: u32, project: Project) -> Self {
		Self {
			sample_rate,
			project,
		}
	}

	pub fn process(&mut self) -> StereoSample {
		StereoSample::from_mono(0.0)
	}
}
