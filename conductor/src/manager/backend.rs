use super::AudioManagerSettings;
use crate::{
	command::{Command, SoundCommand},
	sound::{Sound, SoundId},
	stereo_sample::StereoSample,
};
use indexmap::IndexMap;
use ringbuf::Consumer;

pub(crate) struct Backend {
	sounds: IndexMap<SoundId, Sound>,
	command_consumer: Consumer<Command>,
}

impl Backend {
	pub fn new(settings: AudioManagerSettings, command_consumer: Consumer<Command>) -> Self {
		Self {
			sounds: IndexMap::with_capacity(settings.num_sounds),
			command_consumer,
		}
	}

	fn process_commands(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::Sound(command) => match command {
					SoundCommand::LoadSound(id, sound) => {
						self.sounds.insert(id, sound);
					}
				},
			}
		}
	}

	pub fn process(&mut self) -> StereoSample {
		self.process_commands();
		StereoSample::from_mono(0.0)
	}
}
