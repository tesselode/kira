use atomic::Ordering;
use basedrop::Shared;

use super::{InstanceController, InstancePlaybackState};

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

	pub fn pause(&mut self) {
		self.controller.pause();
	}

	pub fn resume(&mut self) {
		self.controller.resume();
	}

	pub fn stop(&mut self) {
		self.controller.stop();
	}
}
