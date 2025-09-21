use std::sync::Arc;

use crate::modulator::{Modulator, ModulatorBuilder, ModulatorId};

use super::{Tweener, TweenerHandle, TweenerShared, command_writers_and_readers};

/// Configures a tweener.
pub struct TweenerBuilder {
	/// The initial value of the tweener.
	pub initial_value: f64,
}

impl ModulatorBuilder for TweenerBuilder {
	type Handle = TweenerHandle;

	fn build(self, id: ModulatorId) -> (Box<dyn Modulator>, Self::Handle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		let shared = Arc::new(TweenerShared::new());
		(
			Box::new(Tweener::new(
				self.initial_value,
				command_readers,
				shared.clone(),
			)),
			TweenerHandle {
				id,
				command_writers,
				shared,
			},
		)
	}
}
