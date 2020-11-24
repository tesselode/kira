use crate::{frame::Frame, parameter::Parameters};

use super::effect::{Effect, EffectSettings};

#[derive(Debug)]
pub(crate) struct EffectSlot {
	effect: Box<dyn Effect>,
	enabled: bool,
}

impl EffectSlot {
	pub fn new(effect: Box<dyn Effect>, settings: EffectSettings) -> Self {
		Self {
			effect,
			enabled: settings.enabled,
		}
	}

	pub(super) fn process(&mut self, dt: f64, input: Frame, parameters: &Parameters) -> Frame {
		if self.enabled {
			self.effect.process(dt, input, parameters)
		} else {
			input
		}
	}
}
