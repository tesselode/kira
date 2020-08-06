use crate::{sound::Sound, time::Time};
use std::{error::Error, path::Path};

#[derive(Default)]
pub struct SoundSettings {
	pub tempo: Option<f32>,
	pub default_loop_start: Option<Time>,
	pub default_loop_end: Option<Time>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SoundId {
	index: usize,
}

pub struct Project {
	sounds: Vec<Sound>,
}

impl Project {
	pub fn new() -> Self {
		Self { sounds: vec![] }
	}

	pub fn load_sound(
		&mut self,
		path: &Path,
		settings: SoundSettings,
	) -> Result<SoundId, Box<dyn Error>> {
		let id = SoundId {
			index: self.sounds.len(),
		};
		self.sounds.push(Sound::from_ogg_file(path, settings)?);
		Ok(id)
	}

	pub(crate) fn get_sound(&self, id: SoundId) -> &Sound {
		&self.sounds[id.index]
	}
}
