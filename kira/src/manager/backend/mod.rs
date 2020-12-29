mod instances;
mod mixer;
mod sequences;
mod streams;

use self::mixer::Mixer;

use super::AudioManagerSettings;
use crate::{
	arrangement::{Arrangement, ArrangementId},
	command::{Command, ResourceCommand},
	frame::Frame,
	group::groups::Groups,
	metronome::Metronomes,
	parameter::Parameters,
	playable::Playable,
	resource::Resource,
	sound::{Sound, SoundId},
};
use flume::{Receiver, Sender};
use indexmap::IndexMap;
use instances::Instances;
use sequences::Sequences;
use streams::Streams;

pub(crate) struct BackendThreadChannels {
	pub command_receiver: Receiver<Command>,
	pub resources_to_unload_sender: Sender<Resource>,
}

pub struct Backend {
	dt: f64,
	sounds: IndexMap<SoundId, Sound>,
	arrangements: IndexMap<ArrangementId, Arrangement>,
	command_queue: Vec<Command>,
	thread_channels: BackendThreadChannels,
	metronomes: Metronomes,
	parameters: Parameters,
	instances: Instances,
	sequences: Sequences,
	mixer: Mixer,
	groups: Groups,
	streams: Streams,
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
			metronomes: Metronomes::new(settings.num_metronomes),
			instances: Instances::new(settings.num_instances),
			sequences: Sequences::new(settings.num_sequences, settings.num_commands),
			mixer: Mixer::new(),
			groups: Groups::new(settings.num_groups),
			streams: Streams::new(settings.num_streams),
		}
	}

	fn process_commands(&mut self) {
		self.command_queue
			.extend(self.thread_channels.command_receiver.try_iter());
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
							self.thread_channels
								.resources_to_unload_sender
								.try_send(Resource::Sound(sound))
								.ok();
						}
					}
					ResourceCommand::AddArrangement(id, arrangement) => {
						self.arrangements.insert(id, arrangement);
					}
					ResourceCommand::RemoveArrangement(id) => {
						self.instances
							.stop_instances_of(Playable::Arrangement(id), Default::default());
						if let Some(arrangement) = self.arrangements.remove(&id) {
							self.thread_channels
								.resources_to_unload_sender
								.try_send(Resource::Arrangement(arrangement))
								.ok();
						}
					}
				},
				Command::Metronome(command) => {
					self.metronomes.run_command(
						command,
						&mut self.thread_channels.resources_to_unload_sender,
					);
				}
				Command::Instance(command) => {
					self.instances.run_command(
						command,
						&mut self.sounds,
						&mut self.arrangements,
						&self.groups,
					);
				}
				Command::Sequence(command) => {
					self.sequences.run_command(command, &self.groups);
				}
				Command::Mixer(command) => {
					self.mixer.run_command(
						command,
						&mut self.thread_channels.resources_to_unload_sender,
					);
				}
				Command::Parameter(command) => {
					self.parameters.run_command(command);
				}
				Command::Group(command) => {
					if let Some(group) = self.groups.run_command(command) {
						self.thread_channels
							.resources_to_unload_sender
							.try_send(Resource::Group(group))
							.ok();
					}
				}
				Command::Stream(command) => {
					self.streams.run_command(
						command,
						&mut self.thread_channels.resources_to_unload_sender,
					);
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

	fn update_sequences(&mut self) {
		for command in self.sequences.update(
			self.dt,
			&self.metronomes,
			&mut self.thread_channels.resources_to_unload_sender,
		) {
			self.command_queue.push(command.into());
		}
	}

	pub fn process(&mut self) -> Frame {
		self.process_commands();
		self.parameters.update(self.dt);
		self.update_sounds();
		self.update_arrangements();
		self.metronomes.update(self.dt, &self.parameters);
		self.update_sequences();
		self.streams.process(self.dt, &mut self.mixer);
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
