use std::sync::Arc;

use crate::sound::Shared;

pub struct StreamingSoundHandle {
	shared: Arc<Shared>,
}

impl StreamingSoundHandle {
	pub(crate) fn new(shared: Arc<Shared>) -> Self {
		Self { shared }
	}

	pub fn position(&self) -> f64 {
		self.shared.position()
	}
}
