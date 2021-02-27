mod instances;
mod mixer;
mod sequences;
mod streams;

use self::mixer::Mixer;

use super::AudioManagerSettings;
use crate::{
	command::Command, frame::Frame, group::groups::Groups, metronome::Metronomes,
	parameter::Parameters, playable::Playables, static_container::vec::StaticVec,
};
use instances::Instances;
use ringbuf::Consumer;
use sequences::Sequences;
use streams::Streams;

/// Processes audio on the audio thread.
pub struct Backend {
	dt: f64,
	playables: Playables,
	command_queue: StaticVec<Command>,
	command_consumer: Consumer<Command>,
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
		command_consumer: Consumer<Command>,
	) -> Self {
		Self {
			dt: 1.0 / sample_rate as f64,
			playables: Playables::new(settings.num_sounds, settings.num_arrangements),
			command_queue: StaticVec::new(settings.num_commands),
			command_consumer,
			parameters: Parameters::new(settings.num_parameters),
			metronomes: Metronomes::new(settings.num_metronomes),
			instances: Instances::new(settings.num_instances),
			sequences: Sequences::new(settings.num_sequences, settings.num_commands),
			mixer: Mixer::new(settings.num_sub_tracks, settings.num_send_tracks),
			groups: Groups::new(settings.num_groups),
			streams: Streams::new(settings.num_streams),
		}
	}

	fn process_commands(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			// TODO: find a way to avoid sharing the command queue
			// between user-called functions and sequence-produced
			// commands. I don't want sequence commands cutting
			// into the capacity of the command queue
			self.command_queue.try_push(command).ok();
		}
		for command in self.command_queue.drain(..) {
			match command {
				Command::Resource(command) => {
					self.playables.run_command(command);
				}
				Command::Metronome(command) => {
					self.metronomes.run_command(command);
				}
				Command::Instance(command) => {
					self.instances
						.run_command(command, &mut self.playables, &self.groups);
				}
				Command::Sequence(command) => {
					self.sequences.run_command(command, &self.groups);
				}
				Command::Mixer(command) => {
					self.mixer.run_command(command);
				}
				Command::Parameter(command) => {
					self.parameters.run_command(command);
				}
				Command::Group(command) => {
					self.groups.run_command(command);
				}
				Command::Stream(command) => {
					self.streams.run_command(command);
				}
			}
		}
	}

	fn update_sequences(&mut self) {
		for command in self
			.sequences
			.update(self.dt, &self.playables, &self.metronomes)
		{
			self.command_queue.try_push(command.into()).ok();
		}
	}

	/// Produces a frame of audio.
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
