#[derive(Copy, Clone, Eq, PartialEq)]
pub enum InstanceState {
	Stopped,
	Playing,
}

pub struct Instance {
	duration: f32,
	position: f32,
	state: InstanceState,
}

impl Instance {
	pub fn new(duration: f32) -> Self {
		Self {
			duration,
			position: 0.0,
			state: InstanceState::Stopped,
		}
	}

	pub fn state(&self) -> InstanceState {
		self.state
	}

	pub fn position(&self) -> f32 {
		self.position
	}

	pub fn play(&mut self) {
		self.position = 0.0;
		self.state = InstanceState::Playing;
	}

	pub fn update(&mut self, dt: f32) -> Option<f32> {
		match self.state {
			InstanceState::Stopped => None,
			InstanceState::Playing => {
				let position = self.position;
				self.position += dt;
				if self.position >= self.duration {
					self.position = self.duration;
					self.state = InstanceState::Stopped;
				}
				Some(position)
			}
		}
	}
}
