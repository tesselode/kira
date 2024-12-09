use crate::{resources::Resources, Frame, INTERNAL_BUFFER_SIZE};

pub struct Renderer {
	dt: f64,
	resources: Resources,
}

impl Renderer {
	pub fn new(sample_rate: u32, resources: Resources) -> Self {
		Self {
			dt: 1.0 / sample_rate as f64,
			resources,
		}
	}

	/// Called by the backend when the sample rate of the
	/// audio output changes.
	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.dt = 1.0 / sample_rate as f64;
	}

	/// Called by the backend when it's time to process
	/// a new batch of samples.
	pub fn on_start_processing(&mut self) {
		self.resources.sounds.on_start_processing();
	}

	pub fn process(&mut self) -> [Frame; INTERNAL_BUFFER_SIZE] {
		self.resources.sounds.process(self.dt)
	}
}
