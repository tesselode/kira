use crate::{effect::EffectBuilder, tween::Value, Panning};

use super::{command_writers_and_readers, PanningControl, PanningControlHandle};

/// Configures a panning control effect.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PanningControlBuilder(pub Value<Panning>);

impl Default for PanningControlBuilder {
	fn default() -> Self {
		Self(Value::Fixed(Panning::CENTER))
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
