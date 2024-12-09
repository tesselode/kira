use std::f64::consts::TAU;

use crate::{Frame, INTERNAL_BUFFER_SIZE};

pub struct Renderer {
	dt: f64,
	phase: f64,
}

impl Renderer {
	pub fn new(sample_rate: u32) -> Self {
		Self {
			dt: 1.0 / sample_rate as f64,
			phase: 0.0,
		}
	}

	/// Called by the backend when the sample rate of the
	/// audio output changes.
	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.dt = 1.0 / sample_rate as f64;
	}

	/// Called by the backend when it's time to process
	/// a new batch of samples.
	pub fn on_start_processing(&mut self) {}

	pub fn process(&mut self) -> [Frame; INTERNAL_BUFFER_SIZE] {
		let mut frames = [Frame::ZERO; INTERNAL_BUFFER_SIZE];
		for frame in &mut frames {
			let out = Frame::from_mono(0.25 * (self.phase * TAU).sin() as f32);
			self.phase += 440.0 * self.dt;
			self.phase %= 1.0;
			*frame = out;
		}
		frames
	}
}
