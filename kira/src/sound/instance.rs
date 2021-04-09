use std::sync::Arc;

use crate::Frame;

use super::Sound;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceState {
	Playing,
	Stopped,
}

pub(crate) struct Instance {
	sound: Arc<Sound>,
	state: InstanceState,
	playback_position: f64,
}

impl Instance {
	pub fn new(sound: Arc<Sound>) -> Self {
		Self {
			sound,
			state: InstanceState::Playing,
			playback_position: 0.0,
		}
	}

	pub fn state(&self) -> InstanceState {
		self.state
	}

	pub fn process(&mut self, dt: f64) -> Frame {
		match self.state {
			InstanceState::Playing => {
				let output = self.sound.get_frame_at_position(self.playback_position);
				self.playback_position += dt;
				if self.playback_position > self.sound.duration() {
					self.state = InstanceState::Stopped;
				}
				output
			}
			InstanceState::Stopped => Frame::from_mono(0.0),
		}
	}
}
