use crate::modulator::{Modulator, ModulatorBuilder, ModulatorId};

use super::{Tweener, TweenerHandle};

pub struct TweenerBuilder {
	pub initial_value: f64,
}

impl ModulatorBuilder for TweenerBuilder {
	type Handle = TweenerHandle;

	fn build(self, id: ModulatorId) -> (Box<dyn Modulator>, Self::Handle) {
		(
			Box::new(Tweener::new(self.initial_value)),
			TweenerHandle { id },
		)
	}
}
