mod instances;
mod mixer;
mod sequences;

use self::mixer::Mixer;

use super::{AudioManagerSettings, Event};
use crate::{
	arrangement::{Arrangement, ArrangementId},
	command::{Command, ResourceCommand},
	frame::Frame,
	metronome::Metronome,
	mixer::effect_slot::EffectSlot,
	mixer::Track,
	parameter::Parameters,
	playable::Playable,
	sequence::SequenceInstance,
	sound::{Sound, SoundId},
};
use indexmap::IndexMap;
use instances::Instances;
use ringbuf::{Consumer, Producer};
use sequences::Sequences;

pub(crate) struct BackendThreadChannels {
	pub command_consumer: Consumer<Command>,
	pub event_producer: Producer<Event>,
	pub sounds_to_unload_producer: Producer<Sound>,
	pub arrangements_to_unload_producer: Producer<Arrangement>,
	pub sequence_instances_to_unload_producer: Producer<SequenceInstance>,
	pub tracks_to_unload_producer: Producer<Track>,
	pub effect_slots_to_unload_producer: Producer<EffectSlot>,
}

pub struct Backend {
	dt: f64,
	sounds: IndexMap<SoundId, Sound>,
	arrangements: IndexMap<ArrangementId, Arrangement>,
	command_queue: Vec<Command>,
	thread_channels: BackendThreadChannels,
	metronome: Metronome,
	parameters: Parameters,
	instances: Instances,
	sequences: Sequences,
	mixer: Mixer,
}

impl Backend {
	pub(crate) fn new(
		sample_rate: u32,
		settings: AudioManagerSettings,
		thread_channels: BackendThreadChannels,
	) -> Self {
		Self {
			dt: 1.0 / sample_rate as f64,
			sounds: IndexMap::with_capacity(settings.num_sounds),
			arrangements: IndexMap::with_capacity(settings.num_arrangements),
			command_queue: Vec::with_capacity(settings.num_commands),
			thread_channels,
			parameters: Parameters::new(settings.num_parameters),
			metronome: Metronome::new(settings.metronome_settings),
			instances: Instances::new(settings.num_instances),
			sequences: Sequences::new(settings.num_sequences, settings.num_commands),
			mixer: Mixer::new(),
		}
	}

	fn process_commands(&mut self) {
		while let Some(command) = self.thread_channels.command_consumer.pop() {
			self.command_queue.push(command);
		}
		for command in self.command_queue.drain(..) {
			match command {
				Command::Resource(command) => match command {
					ResourceCommand::AddSound(id, sound) => {
						self.sounds.insert(id, sound);
					}
					ResourceCommand::RemoveSound(id) => {
						self.instances
							.stop_instances_of(Playable::Sound(id), Default::default());
						if let Some(sound) = self.sounds.remove(&id) {
							match self.thread_channels.sounds_to_unload_producer.push(sound) {
								_ => {}
							}
						}
					}
					ResourceCommand::AddArrangement(id, arrangement) => {
						self.arrangements.insert(id, arrangement);
					}
					ResourceCommand::RemoveArrangement(id) => {
						self.instances
							.stop_instances_of(Playable::Arrangement(id), Default::default());
						if let Some(arrangement) = self.arrangements.remove(&id) {
							match self
								.thread_channels
								.arrangements_to_unload_producer
								.push(arrangement)
							{
								_ => {}
							}
						}
					}
				},
				Command::Metronome(command) => {
					self.metronome.run_command(command);
				}
				Command::Instance(command) => {
					self.instances
						.run_command(command, &mut self.sounds, &mut self.arrangements);
				}
				Command::Sequence(command) => {
					self.sequences.run_command(command);
				}
				Command::Mixer(command) => {
					self.mixer.run_command(
						command,
						&mut self.thread_channels.tracks_to_unload_producer,
						&mut self.thread_channels.effect_slots_to_unload_producer,
					);
				}
				Command::Parameter(command) => {
					self.parameters.run_command(command);
				}
			}
		}
	}

	fn update_sounds(&mut self) {
		for (_, sound) in &mut self.sounds {
			sound.update_cooldown(self.dt);
		}
	}

	fn update_arrangements(&mut self) {
		for (_, arrangement) in &mut self.arrangements {
			arrangement.update_cooldown(self.dt);
		}
	}

	fn update_metronome(&mut self) {
		for interval in self.metronome.update(self.dt, &self.parameters) {
			match self
				.thread_channels
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
			&mut self.thread_channels.sequence_instances_to_unload_producer,
		) {
			self.command_queue.push(command.into());
		}
	}

	pub fn process(&mut self) -> Frame {
		self.process_commands();
		self.parameters.update(self.dt);
		self.update_sounds();
		self.update_arrangements();
		self.update_metronome();
		self.update_sequences();
		self.instances.process(
			self.dt,
			&self.sounds,
			&self.arrangements,
			&mut self.mixer,
			&self.parameters,
		);
		self.mixer.process(self.dt, &self.parameters)
	}
}
