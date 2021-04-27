use std::sync::Arc;

use super::InstanceController;

pub struct InstanceHandle {
	controller: Arc<InstanceController>,
}

impl InstanceHandle {
	pub(crate) fn new(controller: Arc<InstanceController>) -> Self {
		Self { controller }
	}

	pub fn playback_position(&self) -> f64 {
		self.controller.playback_position()
	}

	pub fn pause(&self) {
		self.controller.pause();
	}

	pub fn resume(&self) {
		self.controller.resume();
	}

	pub fn stop(&self) {
		self.controller.stop();
	}
}
