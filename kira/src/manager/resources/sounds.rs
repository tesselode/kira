use atomic_arena::{Arena, Index};
use ringbuf::Producer;

use crate::{manager::command::SoundCommand, sound::Sound};

pub(crate) struct Sounds {
	pub sounds: Arena<Sound>,
	pub unused_sound_producer: Producer<Sound>,
}

impl Sounds {
	pub fn run_command(&mut self, command: SoundCommand) {
		match command {
			SoundCommand::Add(id, sound) => self
				.sounds
				.insert_with_index(id.0, sound)
				.expect("Sound arena is full"),
		}
	}
}
