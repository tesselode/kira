use std::sync::Arc;

use ringbuf::HeapRb;

use crate::modulator::{Modulator, ModulatorBuilder, ModulatorId};

use super::{Tweener, TweenerHandle, TweenerShared};

const COMMAND_CAPACITY: usize = 8;

/// Configures a tweener.
pub struct TweenerBuilder {
	/// The initial value of the tweener.
	pub initial_value: f64,
}

impl ModulatorBuilder for TweenerBuilder {
	type Handle = TweenerHandle;

	fn build(self, id: ModulatorId) -> (Box<dyn Modulator>, Self::Handle) {
		let (command_producer, command_consumer) = HeapRb::new(COMMAND_CAPACITY).split();
		let shared = Arc::new(TweenerShared::new());
		(
			Box::new(Tweener::new(
				self.initial_value,
				command_consumer,
				shared.clone(),
			)),
			TweenerHandle {
				id,
				command_producer,
				shared,
			},
		)
	}
}
