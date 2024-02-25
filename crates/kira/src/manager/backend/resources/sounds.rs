use atomic_arena::{Arena, Controller};
use ringbuf::HeapProducer;

use crate::{
	clock::clock_info::ClockInfoProvider, manager::command::SoundCommand,
	modulator::value_provider::ModulatorValueProvider, sound::wrapper::SoundWrapper,
	OutputDestination,
};

use super::{mixer::Mixer, spatial_scenes::SpatialScenes};

pub(crate) struct Sounds {
	sound_wrappers: Arena<SoundWrapper>,
	unused_sound_wrapper_producer: HeapProducer<SoundWrapper>,
}

impl Sounds {
	pub fn new(capacity: usize, unused_sound_wrapper_producer: HeapProducer<SoundWrapper>) -> Self {
		Self {
			sound_wrappers: Arena::new(capacity),
			unused_sound_wrapper_producer,
		}
	}

	pub fn controller(&self) -> Controller {
		self.sound_wrappers.controller()
	}

	pub fn on_start_processing(&mut self) {
		for (_, sound_wrapper) in &mut self.sound_wrappers {
			sound_wrapper.on_start_processing();
		}
		self.remove_unused_sounds();
	}

	fn remove_unused_sounds(&mut self) {
		if self.unused_sound_wrapper_producer.is_full() {
			return;
		}
		for (_, sound_wrapper) in self
			.sound_wrappers
			.drain_filter(|sound_wrapper| sound_wrapper.finished())
		{
			if self
				.unused_sound_wrapper_producer
				.push(sound_wrapper)
				.is_err()
			{
				panic!("Unused sound producer is full")
			}
			if self.unused_sound_wrapper_producer.is_full() {
				return;
			}
		}
	}

	pub fn run_command(&mut self, command: SoundCommand) {
		match command {
			SoundCommand::Add(key, sound) => self
				.sound_wrappers
				.insert_with_key(key, sound)
				.expect("Sound arena is full"),
			SoundCommand::Pause(key, tween) => {
				if let Some(sound_wrapper) = self.sound_wrappers.get_mut(key) {
					sound_wrapper.pause(tween);
				}
			}
			SoundCommand::Resume(key, tween) => {
				if let Some(sound_wrapper) = self.sound_wrappers.get_mut(key) {
					sound_wrapper.resume(tween);
				}
			}
			SoundCommand::Stop(key, tween) => {
				if let Some(sound_wrapper) = self.sound_wrappers.get_mut(key) {
					sound_wrapper.stop(tween);
				}
			}
		}
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
		mixer: &mut Mixer,
		scenes: &mut SpatialScenes,
	) {
		for (_, sound_wrapper) in &mut self.sound_wrappers {
			match sound_wrapper.output_destination() {
				OutputDestination::Track(track_id) => {
					if let Some(track) = mixer.track_mut(track_id) {
						track.add_input(sound_wrapper.process(
							dt,
							clock_info_provider,
							modulator_value_provider,
						));
					}
				}
				OutputDestination::Emitter(emitter_id) => {
					if let Some(scene) = scenes.get_mut(emitter_id.scene_id) {
						if let Some(emitter) = scene.emitter_mut(emitter_id) {
							emitter.add_input(sound_wrapper.process(
								dt,
								clock_info_provider,
								modulator_value_provider,
							));
						}
					}
				}
			}
		}
	}
}
