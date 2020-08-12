use crate::sound::{Sound, SoundId};
use std::{collections::HashMap, error::Error, path::Path};

/// Holds sounds and other data used for audio.
pub struct Project {
	pub(crate) sounds: HashMap<SoundId, Sound>,
}

impl Project {
	/// Creates a new, empty project.
	pub fn new() -> Self {
		Self {
			sounds: HashMap::new(),
		}
	}

	/// Loads a sound from a file path.
	///
	/// Returns a handle to the sound. Keep this so you can play the sound later.
	pub fn load_sound(&mut self, path: &Path) -> Result<SoundId, Box<dyn Error>> {
		let sound = Sound::from_ogg_file(path)?;
		let id = SoundId::new(sound.duration());
		self.sounds.insert(id, sound);
		Ok(id)
	}
}
