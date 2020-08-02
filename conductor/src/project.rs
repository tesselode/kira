use crate::sound::Sound;
use std::{error::Error, path::Path};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SoundId {
	pub(crate) index: usize,
}

pub struct Project {
	sounds: Vec<Sound>,
}

impl Project {
	pub fn new() -> Self {
		Self { sounds: vec![] }
	}

	pub fn load_sound(&mut self, path: &Path) -> Result<SoundId, Box<dyn Error>> {
		let id = SoundId {
			index: self.sounds.len(),
		};
		self.sounds.push(Sound::from_ogg_file(path)?);
		Ok(id)
	}
}
