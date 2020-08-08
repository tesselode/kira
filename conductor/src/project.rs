use crate::{
	id::{MetronomeId, SoundId},
	metronome::Metronome,
	sound::Sound,
};
use std::{collections::HashMap, error::Error, path::Path};

pub struct Project {
	pub(crate) sounds: HashMap<SoundId, Sound>,
	pub(crate) metronomes: HashMap<MetronomeId, Metronome>,
}

impl Project {
	pub fn new() -> Self {
		Self {
			sounds: HashMap::new(),
			metronomes: HashMap::new(),
		}
	}

	pub fn load_sound(&mut self, path: &Path) -> Result<SoundId, Box<dyn Error>> {
		let id = SoundId::new();
		self.sounds.insert(id, Sound::from_ogg_file(path)?);
		Ok(id)
	}

	pub fn create_metronome(&mut self, tempo: f32) -> MetronomeId {
		let id = MetronomeId::new();
		self.metronomes.insert(id, Metronome::new(tempo));
		id
	}
}
