use std::sync::atomic::Ordering;

use atomic_arena::{Arena, Index};
use ringbuf::Producer;

use crate::{manager::command::SoundCommand, sound::Sound};

pub(crate) struct Sounds {
	pub sounds: Arena<Sound>,
	pub unused_sound_producer: Producer<Sound>,
}

impl Sounds {
	pub fn on_start_processing(&mut self) {
		if self.unused_sound_producer.is_full() {
			return;
		}
		for (_, sound) in self
			.sounds
			.drain_filter(|sound| sound.shared.removed.load(Ordering::SeqCst))
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
