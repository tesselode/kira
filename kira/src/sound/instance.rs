use std::sync::Arc;

use crate::frame::Frame;

use super::data::SoundData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceState {
	Playing,
	Stopped,
}

pub(crate) struct Instance {
	state: InstanceState,
	position: f64,
}

impl Instance {
	pub fn new() -> Self {
		Self {
			state: InstanceState::Playing,
			position: 0.0,
		}
	}

	pub fn process(&mut self, dt: f64, data: &Arc<dyn SoundData>) -> Frame {
		if let InstanceState::Playing = self.state {
			let out = data.frame_at_position(self.position);
			self.position += dt;
			if self.position > data.duration() {
				self.state = InstanceState::Stopped;
			}
			return out;
		}
		Frame::from_mono(0.0)
	}
}
