mod instances;
mod mixer;
mod sequences;

use self::mixer::Mixer;

use super::{AudioManagerSettings, Event};
use crate::{
	command::{Command, SoundCommand},
	metronome::Metronome,
	parameters::Parameters,
	sequence::Sequence,
	sound::{Sound, SoundId},
	stereo_sample::StereoSample,
};
use indexmap::IndexMap;
use instances::Instances;
use ringbuf::{Consumer, Producer};
use sequences::Sequences;

pub(crate) struct Backend<CustomEvent: Send + 'static + std::fmt::Debug> {
	dt: f64,
	sounds: IndexMap<SoundId, Sound>,
	command_queue: Vec<Command<CustomEvent>>,
	command_consumer: Consumer<Command<CustomEvent>>,
	event_producer: Producer<Event<CustomEvent>>,
	sounds_to_unload_producer: Producer<Sound>,
	sequences_to_unload_producer: Producer<Sequence<CustomEvent>>,
	metronome: Metronome,
	parameters: Parameters,
	instances: Instances,
	sequences: Sequences<CustomEvent>,
	mixer: Mixer,
}

impl<CustomEvent: Copy + Send + 'static + std::fmt::Debug> Backend<CustomEvent> {
	pub fn new(
		sample_rate: u32,
		settings: AudioManagerSettings,
		command_consumer: Consumer<Command<CustomEvent>>,
		event_producer: Producer<Event<CustomEvent>>,
		sounds_to_unload_producer: Producer<Sound>,
		sequences_to_unload_producer: Producer<Sequence<CustomEvent>>,
	) -> Self {
		Self {
			dt: 1.0 / sample_rate as f64,
			sounds: IndexMap::with_capacity(settings.num_sounds),
			command_queue: Vec::with_capacity(settings.num_commands),
			command_consumer,
			event_producer,
			sounds_to_unload_producer,
			sequences_to_unload_producer,
			parameters: Parameters::new(settings.num_parameters),
			metronome: Metronome::new(settings.metronome_settings),
			instances: Instances::new(settings.num_instances),
			sequences: Sequences::new(settings.num_sequences, settings.num_commands),
			mixer: Mixer::new(),
		}
	}

	fn process_commands(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			self.command_queue.push(command);
		}
		for command in self.command_queue.drain(..) {
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
								Err(sound) => {
									self.sounds.insert(id, sound);
								}
							}
						}
					}
				},
				Command::Metronome(command) => {
					self.metronome.run_command(command);
				}
				Command::Instance(command) => {
					self.instances.run_command(command, &mut self.sounds);
				}
				Command::Sequence(command) => {
					self.sequences.run_command(command);
				}
				Command::Mixer(command) => {
					self.mixer.run_command(command);
				}
				Command::Parameter(command) => {
					self.parameters.run_command(command);
				}
				Command::EmitCustomEvent(event) => {
					match self.event_producer.push(Event::Custom(event)) {
						Ok(_) => {}
						Err(_) => {}
					}
				}
			}
		}
	}

	fn update_sounds(&mut self) {
		for (_, sound) in &mut self.sounds {
			sound.update_cooldown(self.dt);
		}
	}

	fn update_metronome(&mut self) {
		for interval in self.metronome.update(self.dt) {
			match self
				.event_producer
				.push(Event::MetronomeIntervalPassed(interval))
			{
				Ok(_) => {}
				Err(_) => {}
			}
		}
	}

	fn update_sequences(&mut self) {
		for command in self.sequences.update(
			self.dt,
			&self.metronome,
			&mut self.sequences_to_unload_producer,
		) {
			self.command_queue.push(command.into());
		}
	}

	pub fn process(&mut self) -> StereoSample {
		self.process_commands();
		self.parameters.update(self.dt);
		self.update_sounds();
		self.update_metronome();
		self.update_sequences();
		self.instances
			.process(self.dt, &self.sounds, &mut self.mixer);
		self.mixer.process(self.dt, &self.parameters)
	}
}
