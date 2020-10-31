use crate::stereo_sample::StereoSample;

use super::{effect::Effect, EffectSettings};

pub struct EffectSlot {
	effect: Box<dyn Effect + Send>,
	enabled: bool,
}

impl EffectSlot {
	pub fn new(effect: Box<dyn Effect + Send>, settings: EffectSettings) -> Self {
		Self {
			effect,
			enabled: settings.enabled,
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
