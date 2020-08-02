use crate::stereo_sample::StereoSample;
use std::f32::consts::PI;

pub struct Backend {
	sample_rate: u32,
	phase: f32,
}

impl Backend {
	pub fn new(sample_rate: u32) -> Self {
		Self {
			sample_rate,
			phase: 0.0,
		}
	}

	pub fn process(&mut self) -> StereoSample {
		self.phase += 440.0 / self.sample_rate as f32;
		self.phase %= 1.0;
		StereoSample::from_mono(0.25 * (self.phase * 2.0 * PI).sin())
	}
}
