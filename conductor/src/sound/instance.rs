#[derive(Copy, Clone, Eq, PartialEq)]
pub enum InstanceState {
	Stopped,
	Playing,
}

pub struct Instance {
	sound_length: usize,
	position: usize,
	state: InstanceState,
}

impl Instance {
	pub fn new(sound_length: usize) -> Self {
		Self {
			sound_length,
			position: 0,
			state: InstanceState::Stopped,
		}
	}

	pub fn state(&self) -> InstanceState {
		self.state
	}

	pub fn position(&self) -> usize {
		self.position
	}

	pub fn play(&mut self) {
		self.position = 0;
		self.state = InstanceState::Playing;
	}

	pub fn update(&mut self) -> Option<usize> {
		match self.state {
			InstanceState::Stopped => None,
			InstanceState::Playing => {
				let position = self.position;
				self.position += 1;
				if self.position == self.sound_length {
					self.state = InstanceState::Stopped;
				}
				Some(position)
			}
		}
	}
}
