use std::sync::atomic::AtomicBool;

use atomig::Ordering;
use basedrop::{Owned, Shared};

use crate::{value::Value, Frame};

use super::effect::Effect;

pub struct EffectSlotState {
	pub enabled: AtomicBool,
}

impl EffectSlotState {
	pub fn new(enabled: bool) -> Self {
		Self {
			enabled: AtomicBool::new(enabled),
		}
	}
}

pub struct EffectSlot {
	effect: Owned<Box<dyn Effect>>,
	mix: Value<f64>,
	state: Shared<EffectSlotState>,
}

impl EffectSlot {
	pub fn new(
		effect: Owned<Box<dyn Effect>>,
		mix: Value<f64>,
		state: Shared<EffectSlotState>,
	) -> Self {
		Self { effect, mix, state }
	}

	pub fn process(&mut self, input: Frame, dt: f64) -> Frame {
		if self.state.enabled.load(Ordering::SeqCst) {
			let wet = self.effect.process(dt, input);
			input + (wet - input) * self.mix.get() as f32
		} else {
			input
		}
	}
}
