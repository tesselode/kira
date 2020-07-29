use crate::manager::PlaySoundSettings;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum InstanceState {
	Stopped,
	Playing,
}

pub struct Instance {
	duration: f32,
	volume: f32,
	pitch: f32,
	position: f32,
	state: InstanceState,
}

impl Instance {
	pub fn new(duration: f32) -> Self {
		Self {
			duration,
			volume: 1.0,
			pitch: 1.0,
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

	pub fn volume(&self) -> f32 {
		self.volume
	}

	pub fn play(&mut self, settings: PlaySoundSettings) {
		self.volume = settings.volume;
		self.pitch = settings.pitch;
		self.position = 0.0;
		self.state = InstanceState::Playing;
	}

	pub fn update(&mut self, dt: f32) -> Option<f32> {
		match self.state {
			InstanceState::Stopped => None,
			InstanceState::Playing => {
				let position = self.position;
				self.position += self.pitch * dt;
				if self.position >= self.duration {
					self.position = self.duration;
					self.state = InstanceState::Stopped;
				}
				Some(position)
			}
		}
	}
}
