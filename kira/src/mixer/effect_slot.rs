use crate::{frame::Frame, parameter::Parameters};

use super::effect::{Effect, EffectId, EffectSettings};

#[derive(Debug)]
pub(crate) struct EffectSlot {
	id: EffectId,
	effect: Box<dyn Effect>,
	enabled: bool,
}

impl EffectSlot {
	pub fn new(effect: Box<dyn Effect>, settings: EffectSettings) -> Self {
		Self {
			id: settings.id,
			effect,
			enabled: settings.enabled,
		}
	}

	pub fn id(&self) -> EffectId {
		self.id
	}

	pub(super) fn process(&mut self, dt: f64, input: Frame, parameters: &Parameters) -> Frame {
		if self.enabled {
			self.effect.process(dt, input, parameters)
		} else {
			input
		}
	}
}
