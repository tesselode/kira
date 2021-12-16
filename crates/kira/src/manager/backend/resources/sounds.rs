use atomic_arena::{Arena, Controller};
use ringbuf::Producer;

use crate::{clock::ClockTime, manager::command::SoundCommand, sound::Sound};

use super::{mixer::Mixer, Parameters};

pub(crate) struct Sounds {
	sounds: Arena<Box<dyn Sound>>,
	unused_sound_producer: Producer<Box<dyn Sound>>,
}

impl Sounds {
	pub fn new(capacity: usize, unused_sound_producer: Producer<Box<dyn Sound>>) -> Self {
		Self {
			sounds: Arena::new(capacity),
			unused_sound_producer,
		}
	}

	pub fn controller(&self) -> Controller {
		self.sounds.controller()
	}

	pub fn on_start_processing(&mut self) {
		for (_, sound) in &mut self.sounds {
			sound.on_start_processing();
		}
		self.remove_unused_sounds();
	}

	fn remove_unused_sounds(&mut self) {
		if self.unused_sound_producer.is_full() {
			return;
		}
		for (_, sound) in self.sounds.drain_filter(|sound| sound.finished()) {
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
			SoundCommand::Add(key, sound) => self
				.sounds
				.insert_with_key(key, sound)
				.expect("Sound arena is full"),
		}
	}

	pub fn process(&mut self, dt: f64, parameters: &Parameters, mixer: &mut Mixer) {
		for (_, sound) in &mut self.sounds {
			if let Some(track) = mixer.track_mut(sound.track()) {
				track.add_input(sound.process(dt, parameters));
			}
		}
	}

	pub fn on_clock_tick(&mut self, time: ClockTime) {
		for (_, sound) in &mut self.sounds {
			sound.on_clock_tick(time);
		}
	}
}
