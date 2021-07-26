use atomic_arena::Index;

use crate::frame::Frame;

use super::{data::SoundData, SoundId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceId(pub(crate) Index);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceState {
	Playing,
	Stopped,
}

pub(crate) struct Instance {
	sound_id: SoundId,
	state: InstanceState,
	position: f64,
}

impl Instance {
	pub fn new(sound_id: SoundId) -> Self {
		Self {
			sound_id,
			state: InstanceState::Playing,
			position: 0.0,
		}
	}

	pub fn sound_id(&self) -> SoundId {
		self.sound_id
	}

	pub fn state(&self) -> InstanceState {
		self.state
	}

	pub fn process(&mut self, dt: f64, sound_data: &Box<dyn SoundData>) -> Frame {
		if let InstanceState::Playing = self.state {
			let out = sound_data.frame_at_position(self.position);
			self.position += dt;
			if self.position > sound_data.duration() {
				self.state = InstanceState::Stopped;
			}
			return out;
		}
		Frame::from_mono(0.0)
	}
}
