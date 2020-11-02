use crate::{manager::backend::parameters::Parameters, stereo_sample::StereoSample};

use super::{effect::Effect, EffectSettings};

pub(crate) struct EffectSlot {
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

	pub(super) fn process(
		&mut self,
		dt: f64,
		input: StereoSample,
		parameters: &Parameters,
	) -> StereoSample {
		if self.enabled {
			self.effect.process(dt, input, parameters)
		} else {
			input
		}
	}
}
