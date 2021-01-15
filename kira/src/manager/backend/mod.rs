mod instances;
mod mixer;
mod sequences;
mod streams;

use self::mixer::Mixer;

use super::AudioManagerSettings;
use crate::{
	command::Command, frame::Frame, group::groups::Groups, metronome::Metronomes,
	parameter::Parameters, playable::Playables, resource::Resource,
};
use flume::{Receiver, Sender};
use instances::Instances;
use sequences::Sequences;
use streams::Streams;

pub struct Backend {
	dt: f64,
	playables: Playables,
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
			playables: Playables::new(settings.num_sounds, settings.num_arrangements),
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
				Command::Resource(command) => {
					self.playables.run_command(command, &mut self.unloader);
				}
				Command::Metronome(command) => {
					self.metronomes.run_command(command, &mut self.unloader);
				}
				Command::Instance(command) => {
					self.instances
						.run_command(command, &mut self.playables, &self.groups);
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
					self.groups.run_command(command, &mut self.unloader);
				}
				Command::Stream(command) => {
					self.streams.run_command(command, &mut self.unloader);
				}
			}
		}
	}

	fn update_sequences(&mut self) {
		for command in self.sequences.update(
			self.dt,
			&self.playables,
			&self.metronomes,
			&mut self.unloader,
		) {
			self.command_queue.push(command.into());
		}
	}

	pub fn process(&mut self) -> Frame {
		self.process_commands();
		self.parameters.update(self.dt);
		self.playables.update(self.dt);
		self.metronomes.update(self.dt, &self.parameters);
		self.update_sequences();
		self.streams.process(self.dt, &mut self.mixer);
		self.instances
			.process(self.dt, &self.playables, &mut self.mixer, &self.parameters);
		self.mixer.process(self.dt, &self.parameters)
	}
}
