use atomig::Ordering;
use basedrop::Shared;

use super::InstanceController;

pub struct InstanceHandle {
	controller: Shared<InstanceController>,
}

impl InstanceHandle {
	pub(crate) fn new(controller: Shared<InstanceController>) -> Self {
		Self { controller }
	}

	pub fn playback_position(&self) -> f64 {
		self.controller.playback_position.load(Ordering::Relaxed)
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
