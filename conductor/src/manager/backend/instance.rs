use crate::{manager::InstanceSettings, project::SoundId};

pub struct Instance {
	pub sound_id: SoundId,
	pub position: f32,
	pub volume: f32,
	pub pitch: f32,
}

impl Instance {
	pub fn new(sound_id: SoundId, settings: InstanceSettings) -> Self {
		Self {
			sound_id,
			position: 0.0,
			volume: settings.volume,
			pitch: settings.pitch,
		}
	}
}
