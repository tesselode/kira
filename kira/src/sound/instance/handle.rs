use basedrop::Shared;

use super::Instance;

pub struct InstanceHandle {
	instance: Shared<Instance>,
}

impl InstanceHandle {
	pub(crate) fn new(instance: Shared<Instance>) -> Self {
		Self { instance }
	}

	pub fn playback_position(&self) -> f64 {
		self.instance.playback_position()
	}

	pub fn pause(&self) {
		self.instance.pause();
	}

	pub fn resume(&self) {
		self.instance.resume();
	}

	pub fn stop(&self) {
		self.instance.stop();
	}
}
