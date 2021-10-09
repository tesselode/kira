use atomic_arena::{Arena, Controller};
use ringbuf::Producer;

use crate::{
	manager::command::SoundCommand,
	sound::{wrapper::SoundWrapper, SoundId},
};

pub(crate) struct Sounds {
	sounds: Arena<SoundWrapper>,
	unused_sound_producer: Producer<SoundWrapper>,
}

impl Sounds {
	pub fn new(capacity: usize, unused_sound_producer: Producer<SoundWrapper>) -> Self {
		Self {
			sounds: Arena::new(capacity),
			unused_sound_producer,
		}
	}

	pub fn controller(&self) -> Controller {
		self.sounds.controller()
	}

	pub fn get_mut(&mut self, id: SoundId) -> Option<&mut SoundWrapper> {
		self.sounds.get_mut(id.0)
	}

	pub fn on_start_processing(&mut self) {
		if self.unused_sound_producer.is_full() {
			return;
		}
		for (_, sound) in self
			.sounds
			.drain_filter(|sound| sound.shared.is_marked_for_removal())
		{
			if self.unused_sound_producer.push(sound).is_err() {
				panic!("Unused sound producer is full")
			}
			if self.unused_sound_producer.is_full() {
				return;
			}
		}
	}

	pub fn run_command(&mut self, command: SoundCommand) {
		match command {
			SoundCommand::Add(id, sound) => self
				.sounds
				.insert_with_index(id.0, sound)
				.expect("Sound arena is full"),
		}
	}
}
