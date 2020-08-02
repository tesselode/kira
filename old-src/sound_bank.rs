use crate::sound::Sound;
use std::{error::Error, path::Path};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct SoundId {
	pub(crate) index: usize,
}

pub struct SoundBank {
	pub(crate) sounds: Vec<Sound>,
	next_id: usize,
}

impl SoundBank {
	pub fn new() -> Self {
		Self {
			sounds: vec![],
			next_id: 0,
		}
	}

	pub fn load(&mut self, path: &Path) -> Result<SoundId, Box<dyn Error>> {
		let id = SoundId {
			index: self.next_id,
		};
		self.next_id += 1;
		self.sounds.push(Sound::from_ogg_file(path)?);
		Ok(id)
	}
}
