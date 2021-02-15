use basedrop::Owned;

use crate::{frame::Frame, parameter::Parameters, CachedValue};

use super::effect::{Effect, EffectSettings};

pub(crate) struct EffectSlot {
	effect: Owned<Box<dyn Effect>>,
	pub enabled: bool,
	pub mix: CachedValue<f64>,
}

impl EffectSlot {
	pub fn new(effect: Owned<Box<dyn Effect>>, settings: EffectSettings) -> Self {
		Self {
			effect,
			enabled: settings.enabled,
			mix: CachedValue::new(settings.mix, 1.0),
		}
	}

	pub(super) fn process(&mut self, dt: f64, input: Frame, parameters: &Parameters) -> Frame {
		self.mix.update(parameters);
		if self.enabled {
			let wet = self.effect.process(dt, input, parameters);
			input + (wet - input) * self.mix.value() as f32
		} else {
			input
		}
	}
}
