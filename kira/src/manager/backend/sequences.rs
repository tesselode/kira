use crate::{
	command::{Command, InstanceCommand, MetronomeCommand, ParameterCommand, SequenceCommand},
	group::groups::Groups,
	instance::Instance,
	metronome::Metronomes,
	playable::Playables,
	resource::Resource,
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

	fn start_sequence_instance(
		&mut self,
		id: SequenceInstanceId,
		mut instance: SequenceInstance,
	) -> Option<SequenceInstance> {
		instance.start();
		self.sequence_instances.insert(id, instance)
	}

	pub fn run_command(
		&mut self,
		command: SequenceCommand,
		groups: &Groups,
		unloader: &mut Sender<Resource>,
	) {
		match command {
			SequenceCommand::StartSequenceInstance(id, instance) => {
				if let Some(instance) = self.start_sequence_instance(id, instance) {
					unloader.try_send(Resource::SequenceInstance(instance)).ok();
				}
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
		playables: &Playables,
		metronomes: &Metronomes,
		unloader: &mut Sender<Resource>,
	) -> Drain<Command> {
		// update sequences and process their commands
		for (id, sequence_instance) in &mut self.sequence_instances {
			sequence_instance.update(dt, metronomes, &mut self.sequence_output_command_queue);
			// convert sequence commands to commands that can be consumed
			// by the backend
			for command in self.sequence_output_command_queue.drain(..) {
				match command {
					SequenceOutputCommand::PlaySound(playable_id, settings) => {
						if let Some(playable) = playables.playable(playable_id) {
							let instance_id = settings.id;
							self.output_command_queue.push(Command::Instance(
								InstanceCommand::Play(
									instance_id,
									Instance::new(
										playable_id,
										playable.duration(),
										Some(*id),
										settings.into_internal(
											playable.duration(),
											playable.default_loop_start(),
											playable.default_track(),
										),
									),
								),
							))
						}
					}
					SequenceOutputCommand::SetInstanceVolume(id, volume) => {
						self.output_command_queue.push(Command::Instance(
							InstanceCommand::SetInstanceVolume(id, volume),
						))
					}
					SequenceOutputCommand::SetInstancePitch(id, pitch) => {
						self.output_command_queue.push(Command::Instance(
							InstanceCommand::SetInstancePitch(id, pitch),
						))
					}
					SequenceOutputCommand::SetInstancePanning(id, panning) => {
						self.output_command_queue.push(Command::Instance(
							InstanceCommand::SetInstancePanning(id, panning),
						))
					}
					SequenceOutputCommand::PauseInstance(id, settings) => {
						self.output_command_queue.push(Command::Instance(
							InstanceCommand::PauseInstance(id, settings),
						))
					}
					SequenceOutputCommand::ResumeInstance(id, settings) => {
						self.output_command_queue.push(Command::Instance(
							InstanceCommand::ResumeInstance(id, settings),
						))
					}
					SequenceOutputCommand::StopInstance(id, settings) => {
						self.output_command_queue.push(Command::Instance(
							InstanceCommand::StopInstance(id, settings),
						))
					}
					SequenceOutputCommand::PauseInstancesOf(id, settings) => {
						self.output_command_queue.push(Command::Instance(
							InstanceCommand::PauseInstancesOf(id, settings),
						))
					}
					SequenceOutputCommand::ResumeInstancesOf(id, settings) => {
						self.output_command_queue.push(Command::Instance(
							InstanceCommand::ResumeInstancesOf(id, settings),
						))
					}
					SequenceOutputCommand::StopInstancesOf(id, settings) => {
						self.output_command_queue.push(Command::Instance(
							InstanceCommand::StopInstancesOf(id, settings),
						))
					}
					SequenceOutputCommand::PauseSequence(id) => self.output_command_queue.push(
						Command::Sequence(SequenceCommand::PauseSequenceInstance(id)),
					),
					SequenceOutputCommand::ResumeSequence(id) => self.output_command_queue.push(
						Command::Sequence(SequenceCommand::ResumeSequenceInstance(id)),
					),
					SequenceOutputCommand::StopSequence(id) => self
						.output_command_queue
						.push(Command::Sequence(SequenceCommand::StopSequenceInstance(id))),
					SequenceOutputCommand::PauseInstancesOfSequence(id, settings) => {
						self.output_command_queue.push(Command::Instance(
							InstanceCommand::PauseInstancesOfSequence(id, settings),
						))
					}
					SequenceOutputCommand::ResumeInstancesOfSequence(id, settings) => {
						self.output_command_queue.push(Command::Instance(
							InstanceCommand::ResumeInstancesOfSequence(id, settings),
						))
					}
					SequenceOutputCommand::StopInstancesOfSequence(id, settings) => {
						self.output_command_queue.push(Command::Instance(
							InstanceCommand::StopInstancesOfSequence(id, settings),
						))
					}
					SequenceOutputCommand::SetMetronomeTempo(id, tempo) => {
						self.output_command_queue.push(Command::Metronome(
							MetronomeCommand::SetMetronomeTempo(id, tempo),
						))
					}
					SequenceOutputCommand::StartMetronome(id) => self
						.output_command_queue
						.push(Command::Metronome(MetronomeCommand::StartMetronome(id))),
					SequenceOutputCommand::PauseMetronome(id) => self
						.output_command_queue
						.push(Command::Metronome(MetronomeCommand::PauseMetronome(id))),
					SequenceOutputCommand::StopMetronome(id) => self
						.output_command_queue
						.push(Command::Metronome(MetronomeCommand::StopMetronome(id))),
					SequenceOutputCommand::SetParameter(id, target, tween) => {
						self.output_command_queue.push(Command::Parameter(
							ParameterCommand::SetParameter(id, target, tween),
						))
					}
				}
			}
			if sequence_instance.finished() {
				self.sequence_instances_to_remove.push(*id);
			}
		}
		// remove finished sequences
		for id in self.sequence_instances_to_remove.drain(..) {
			let instance = self.sequence_instances.remove(&id).unwrap();
			unloader.try_send(Resource::SequenceInstance(instance)).ok();
		}
		self.output_command_queue.drain(..)
	}
}
