use atomic_arena::{Arena, Controller};
use ringbuf::HeapProducer;

use crate::{
	clock::clock_info::ClockInfoProvider, manager::command::SoundCommand, sound::Sound,
	OutputDestination,
};

use super::{mixer::Mixer, spatial_scenes::SpatialScenes};

pub(crate) struct Sounds {
	sounds: Arena<Box<dyn Sound>>,
	unused_sound_producer: HeapProducer<Box<dyn Sound>>,
}

impl Sounds {
	pub fn new(capacity: usize, unused_sound_producer: HeapProducer<Box<dyn Sound>>) -> Self {
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

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		mixer: &mut Mixer,
		scenes: &mut SpatialScenes,
	) {
		for (_, sound) in &mut self.sounds {
			match sound.output_destination() {
				OutputDestination::Track(track_id) => {
					if let Some(track) = mixer.track_mut(track_id) {
						track.add_input(sound.process(dt, clock_info_provider));
					}
				}
				OutputDestination::Emitter(emitter_id) => {
					if let Some(scene) = scenes.get_mut(emitter_id.scene_id) {
						if let Some(emitter) = scene.emitter_mut(emitter_id) {
							emitter.add_input(sound.process(dt, clock_info_provider));
						}
					}
				}
			}
		}
	}
}
