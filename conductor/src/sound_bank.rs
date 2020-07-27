use crate::sound::Sound;
use std::{collections::HashMap, error::Error, path::Path};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct SoundId {
	pub(crate) id: usize,
}

pub struct SoundBank {
	pub(crate) sounds: HashMap<SoundId, Sound>,
	next_id: usize,
}

impl SoundBank {
	pub fn new() -> Self {
		Self {
			sounds: HashMap::new(),
			next_id: 0,
		}
	}

	pub fn load(&mut self, path: &Path) -> Result<SoundId, Box<dyn Error>> {
		let id = SoundId { id: self.next_id };
		self.next_id += 1;
		self.sounds.insert(id, Sound::from_ogg_file(path)?);
		Ok(id)
	}
}
