use atomic_arena::Index;

use crate::{frame::Frame, manager::resources::sounds::Sounds};

use super::SoundId;

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

	pub fn state(&self) -> InstanceState {
		self.state
	}

	pub fn process(&mut self, dt: f64, sounds: &Sounds) -> Frame {
		let sound = match sounds.get(self.sound_id) {
			Some(sound) => sound,
			None => return Frame::from_mono(0.0),
		};
		if let InstanceState::Playing = self.state {
			let out = sound.data.frame_at_position(self.position);
			self.position += dt;
			if self.position > sound.data.duration() {
				self.state = InstanceState::Stopped;
			}
			return out;
		}
		Frame::from_mono(0.0)
	}
}
