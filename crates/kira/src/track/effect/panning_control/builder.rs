use ringbuf::RingBuffer;

use crate::track::effect::EffectBuilder;

use super::{PanningControl, PanningControlHandle};

const COMMAND_CAPACITY: usize = 8;

/// Configures a panning control effect.
#[derive(Debug, Copy, Clone)]
pub struct PanningControlBuilder(pub f64);

impl Default for PanningControlBuilder {
	fn default() -> Self {
		Self(0.5)
	}
}

impl EffectBuilder for PanningControlBuilder {
	type Handle = PanningControlHandle;

	fn build(self) -> (Box<dyn crate::track::effect::Effect>, Self::Handle) {
		let (command_producer, command_consumer) = RingBuffer::new(COMMAND_CAPACITY).split();
		(
			Box::new(PanningControl::new(self, command_consumer)),
			PanningControlHandle { command_producer },
		)
	}
}
