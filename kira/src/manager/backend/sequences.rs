use crate::{
	command::{Command, InstanceCommand, MetronomeCommand, ParameterCommand, SequenceCommand},
	group::groups::Groups,
	instance::Instance,
	metronome::Metronome,
	sequence::{SequenceInstance, SequenceInstanceId, SequenceOutputCommand},
};
use flume::Sender;
use indexmap::IndexMap;
use std::vec::Drain;

pub(crate) struct Sequences {
	sequence_instances: IndexMap<SequenceInstanceId, SequenceInstance>,
	sequence_instances_to_remove: Vec<SequenceInstanceId>,
	sequence_output_command_queue: Vec<SequenceOutputCommand>,
	output_command_queue: Vec<Command>,
}

impl Sequences {
	pub fn new(sequence_capacity: usize, command_capacity: usize) -> Self {
		Self {
			sequence_instances: IndexMap::with_capacity(sequence_capacity),
			sequence_instances_to_remove: Vec::with_capacity(sequence_capacity),
			sequence_output_command_queue: Vec::with_capacity(command_capacity),
			output_command_queue: Vec::with_capacity(command_capacity),
		}
	}

	fn start_sequence_instance(&mut self, id: SequenceInstanceId, mut instance: SequenceInstance) {
		instance.start();
		self.sequence_instances.insert(id, instance);
	}

	pub fn run_command(&mut self, command: SequenceCommand, groups: &Groups) {
		match command {
			SequenceCommand::StartSequenceInstance(id, instance) => {
				self.start_sequence_instance(id, instance);
			}
			SequenceCommand::MuteSequenceInstance(id) => {
				if let Some(instance) = self.sequence_instances.get_mut(&id) {
					instance.mute();
				}
			}
			SequenceCommand::UnmuteSequenceInstance(id) => {
				if let Some(instance) = self.sequence_instances.get_mut(&id) {
					instance.unmute();
				}
			}
			SequenceCommand::PauseSequenceInstance(id) => {
				if let Some(instance) = self.sequence_instances.get_mut(&id) {
					instance.pause();
				}
			}
			SequenceCommand::ResumeSequenceInstance(id) => {
				if let Some(instance) = self.sequence_instances.get_mut(&id) {
					instance.resume();
				}
			}
			SequenceCommand::StopSequenceInstance(id) => {
				if let Some(instance) = self.sequence_instances.get_mut(&id) {
					instance.stop();
				}
			}
			SequenceCommand::PauseGroup(id) => {
				for (_, instance) in &mut self.sequence_instances {
					if instance.is_in_group(id, groups) {
						instance.pause();
					}
				}
			}
			SequenceCommand::ResumeGroup(id) => {
				for (_, instance) in &mut self.sequence_instances {
					if instance.is_in_group(id, groups) {
						instance.resume();
					}
				}
			}
			SequenceCommand::StopGroup(id) => {
				for (_, instance) in &mut self.sequence_instances {
					if instance.is_in_group(id, groups) {
						instance.stop();
					}
				}
			}
		}
	}

	pub fn update(
		&mut self,
		dt: f64,
		metronome: &Metronome,
		sequences_to_unload_sender: &mut Sender<SequenceInstance>,
	) -> Drain<Command> {
		// update sequences and process their commands
		for (id, sequence_instance) in &mut self.sequence_instances {
			sequence_instance.update(dt, metronome, &mut self.sequence_output_command_queue);
			// convert sequence commands to commands that can be consumed
			// by the backend
			for command in self.sequence_output_command_queue.drain(..) {
				self.output_command_queue.push(match command {
					SequenceOutputCommand::PlaySound(instance_id, playable, settings) => {
						Command::Instance(InstanceCommand::Play(
							instance_id,
							Instance::new(playable, Some(*id), settings),
						))
					}
					SequenceOutputCommand::SetInstanceVolume(id, volume) => {
						Command::Instance(InstanceCommand::SetInstanceVolume(id, volume))
					}
					SequenceOutputCommand::SetInstancePitch(id, pitch) => {
						Command::Instance(InstanceCommand::SetInstancePitch(id, pitch))
					}
					SequenceOutputCommand::SetInstancePanning(id, panning) => {
						Command::Instance(InstanceCommand::SetInstancePanning(id, panning))
					}
					SequenceOutputCommand::PauseInstance(id, settings) => {
						Command::Instance(InstanceCommand::PauseInstance(id, settings))
					}
					SequenceOutputCommand::ResumeInstance(id, settings) => {
						Command::Instance(InstanceCommand::ResumeInstance(id, settings))
					}
					SequenceOutputCommand::StopInstance(id, settings) => {
						Command::Instance(InstanceCommand::StopInstance(id, settings))
					}
					SequenceOutputCommand::PauseInstancesOf(id, settings) => {
						Command::Instance(InstanceCommand::PauseInstancesOf(id, settings))
					}
					SequenceOutputCommand::ResumeInstancesOf(id, settings) => {
						Command::Instance(InstanceCommand::ResumeInstancesOf(id, settings))
					}
					SequenceOutputCommand::StopInstancesOf(id, settings) => {
						Command::Instance(InstanceCommand::StopInstancesOf(id, settings))
					}
					SequenceOutputCommand::PauseSequence(id) => {
						Command::Sequence(SequenceCommand::PauseSequenceInstance(id))
					}
					SequenceOutputCommand::ResumeSequence(id) => {
						Command::Sequence(SequenceCommand::ResumeSequenceInstance(id))
					}
					SequenceOutputCommand::StopSequence(id) => {
						Command::Sequence(SequenceCommand::StopSequenceInstance(id))
					}
					SequenceOutputCommand::PauseInstancesOfSequence(id, settings) => {
						Command::Instance(InstanceCommand::PauseInstancesOfSequence(id, settings))
					}
					SequenceOutputCommand::ResumeInstancesOfSequence(id, settings) => {
						Command::Instance(InstanceCommand::ResumeInstancesOfSequence(id, settings))
					}
					SequenceOutputCommand::StopInstancesOfSequence(id, settings) => {
						Command::Instance(InstanceCommand::StopInstancesOfSequence(id, settings))
					}
					SequenceOutputCommand::SetMetronomeTempo(tempo) => {
						Command::Metronome(MetronomeCommand::SetMetronomeTempo(tempo))
					}
					SequenceOutputCommand::StartMetronome => {
						Command::Metronome(MetronomeCommand::StartMetronome)
					}
					SequenceOutputCommand::PauseMetronome => {
						Command::Metronome(MetronomeCommand::PauseMetronome)
					}
					SequenceOutputCommand::StopMetronome => {
						Command::Metronome(MetronomeCommand::StopMetronome)
					}
					SequenceOutputCommand::SetParameter(id, target, tween) => {
						Command::Parameter(ParameterCommand::SetParameter(id, target, tween))
					}
				});
			}
			if sequence_instance.finished() {
				self.sequence_instances_to_remove.push(*id);
			}
		}
		// remove finished sequences
		for id in self.sequence_instances_to_remove.drain(..) {
			let instance = self.sequence_instances.remove(&id).unwrap();
			sequences_to_unload_sender.try_send(instance).ok();
		}
		self.output_command_queue.drain(..)
	}
}
