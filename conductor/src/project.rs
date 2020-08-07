use crate::{id::SoundId, sound::Sound};
use std::{collections::HashMap, error::Error, path::Path};

pub struct Project {
	sounds: HashMap<SoundId, Sound>,
}

impl Project {
	pub fn new() -> Self {
		Self {
			sounds: HashMap::new(),
		}
	}

	pub fn load_sound(&mut self, path: &Path) -> Result<SoundId, Box<dyn Error>> {
		let id = SoundId::new();
		let sound = Sound::from_ogg_file(path)?;
		self.sounds.insert(id, sound);
		Ok(id)
	}

	pub(crate) fn get_sound(&self, id: &SoundId) -> &Sound {
		self.sounds.get(id).unwrap()
	}
}
