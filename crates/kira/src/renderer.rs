use std::f32::consts::TAU;

const INTERNAL_BUFFER_SIZE: usize = 128;

pub struct Renderer {
	dt: f64,
	phase: f32,
}

impl Renderer {
	pub fn new(sample_rate: u32) -> Self {
		Self {
			dt: 1.0 / sample_rate as f64,
			phase: 0.0,
		}
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.dt = 1.0 / sample_rate as f64;
	}

	pub fn on_start_processing(&mut self) {}

	pub fn process(&mut self, out: &mut [f32], channels: u16) {
		for frame in out.chunks_mut(channels.into()) {
			let sine_out = 0.1 * (self.phase * TAU).sin();
			self.phase += 440.0 * self.dt as f32;
			self.phase %= 1.0;
			for channel in frame {
				*channel = sine_out;
			}
		}
	}
}
