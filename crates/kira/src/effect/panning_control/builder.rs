use crate::{effect::EffectBuilder, tween::Value};

use super::{command_writers_and_readers, PanningControl, PanningControlHandle};

/// Configures a panning control effect.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PanningControlBuilder(pub Value<f64>);

impl Default for PanningControlBuilder {
	fn default() -> Self {
		Self(Value::Fixed(0.5))
	}
}

impl EffectBuilder for PanningControlBuilder {
	type Handle = PanningControlHandle;

	fn build(self) -> (Box<dyn crate::effect::Effect>, Self::Handle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		(
			Box::new(PanningControl::new(self, command_readers)),
			PanningControlHandle { command_writers },
		)
	}
}
