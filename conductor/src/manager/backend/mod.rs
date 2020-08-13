mod instances;

use super::AudioManagerSettings;
use crate::{
	command::{Command, SoundCommand},
	sound::{Sound, SoundId},
	stereo_sample::StereoSample,
};
use indexmap::IndexMap;
use instances::Instances;
use ringbuf::{Consumer, Producer};

pub(crate) struct Backend {
	dt: f32,
	sounds: IndexMap<SoundId, Sound>,
	command_consumer: Consumer<Command>,
	sounds_to_unload_producer: Producer<Sound>,
	instances: Instances,
}

impl Backend {
	pub fn new(
		sample_rate: u32,
		settings: AudioManagerSettings,
		command_consumer: Consumer<Command>,
		sounds_to_unload_producer: Producer<Sound>,
	) -> Self {
		Self {
			dt: 1.0 / sample_rate as f32,
			sounds: IndexMap::with_capacity(settings.num_sounds),
			command_consumer,
			sounds_to_unload_producer,
			instances: Instances::new(settings.num_instances),
		}
	}

	fn process_commands(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::Sound(command) => match command {
					SoundCommand::LoadSound(id, sound) => {
						self.sounds.insert(id, sound);
					}
					SoundCommand::UnloadSound(id) => {
						self.instances.stop_instances_of_sound(id, None);
						if let Some(sound) = self.sounds.remove(&id) {
							match self.sounds_to_unload_producer.push(sound) {
								Ok(_) => {}
								Err(_) => {}
							}
						}
					}
				},
				Command::Instance(command) => {
					self.instances.run_command(command);
				}
			}
		}
	}

	pub fn process(&mut self) -> StereoSample {
		self.process_commands();
		self.instances.process(self.dt, &self.sounds)
	}
}
