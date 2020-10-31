use crate::{effect::Effect, stereo_sample::StereoSample};

pub struct EffectSlot {
	effect: Box<dyn Effect + Send>,
	enabled: bool,
}

impl EffectSlot {
	pub fn new(effect: Box<dyn Effect + Send>) -> Self {
		Self {
			effect,
			enabled: true,
		}
	}

	pub(super) fn process(&mut self, dt: f64, input: StereoSample) -> StereoSample {
		if self.enabled {
			self.effect.process(dt, input)
		} else {
			input
		}
	}
}
