use crate::{project::SoundId, time::Time};

pub struct Looper {
	pub sound_id: SoundId,
	pub start: Time,
	pub end: Time,
	position: f32,
}

impl Looper {
	pub fn new(sound_id: SoundId, start: Time, end: Time) -> Self {
		Self {
			sound_id,
			start,
			end,
			position: 0.0,
		}
	}

	pub fn update(&mut self, dt: f32, tempo: f32) -> bool {
		let start = self.start.in_seconds(tempo);
		let end = self.end.in_seconds(tempo);
		self.position += dt;
		if self.position >= end {
			self.position -= end - start;
			return true;
		}
		false
	}
}
