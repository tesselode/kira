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
	playable::PlayableId,
	resource::Resource,
	sound::{Sound, SoundId},
};
use flume::{Receiver, Sender};
use indexmap::IndexMap;
use instances::Instances;
use sequences::Sequences;
use streams::Streams;

pub struct Backend {
	dt: f64,
	sounds: IndexMap<SoundId, Sound>,
	arrangements: IndexMap<ArrangementId, Arrangement>,
	command_queue: Vec<Command>,
	command_receiver: Receiver<Command>,
	unloader: Sender<Resource>,
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
		command_receiver: Receiver<Command>,
		unloader: Sender<Resource>,
	) -> Self {
		Self {
			dt: 1.0 / sample_rate as f64,
			sounds: IndexMap::with_capacity(settings.num_sounds),
			arrangements: IndexMap::with_capacity(settings.num_arrangements),
			command_queue: Vec::with_capacity(settings.num_commands),
			command_receiver,
			unloader,
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
		self.command_queue.extend(self.command_receiver.try_iter());
		for command in self.command_queue.drain(..) {
			match command {
				Command::Resource(command) => match command {
					ResourceCommand::AddSound(sound) => {
						if let Some(sound) = self.sounds.insert(sound.id(), sound) {
							self.unloader.try_send(Resource::Sound(sound)).ok();
						}
					}
					ResourceCommand::RemoveSound(id) => {
						self.instances
							.stop_instances_of(PlayableId::Sound(id), Default::default());
						if let Some(sound) = self.sounds.remove(&id) {
							self.unloader.try_send(Resource::Sound(sound)).ok();
						}
					}
					ResourceCommand::AddArrangement(arrangement) => {
						if let Some(arrangement) =
							self.arrangements.insert(arrangement.id(), arrangement)
						{
							self.unloader
								.try_send(Resource::Arrangement(arrangement))
								.ok();
						}
					}
					ResourceCommand::RemoveArrangement(id) => {
						self.instances
							.stop_instances_of(PlayableId::Arrangement(id), Default::default());
						if let Some(arrangement) = self.arrangements.remove(&id) {
							self.unloader
								.try_send(Resource::Arrangement(arrangement))
								.ok();
						}
					}
				},
				Command::Metronome(command) => {
					self.metronomes.run_command(command, &mut self.unloader);
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
					self.sequences
						.run_command(command, &self.groups, &mut self.unloader);
				}
				Command::Mixer(command) => {
					self.mixer.run_command(command, &mut self.unloader);
				}
				Command::Parameter(command) => {
					self.parameters.run_command(command);
				}
				Command::Group(command) => {
					if let Some(group) = self.groups.run_command(command) {
						self.unloader.try_send(Resource::Group(group)).ok();
					}
				}
				Command::Stream(command) => {
					self.streams.run_command(command, &mut self.unloader);
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
		for command in self
			.sequences
			.update(self.dt, &self.metronomes, &mut self.unloader)
		{
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
