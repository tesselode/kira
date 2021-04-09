use crate::{
	command::{Command, InstanceCommand, MetronomeCommand, ParameterCommand, SequenceCommand},
	group::groups::Groups,
	instance::Instance,
	metronome::Metronomes,
	playable::Playables,
	sequence::{SequenceInstance, SequenceInstanceId, SequenceOutputCommand},
	static_container::{index_map::StaticIndexMap, vec::StaticVec},
};
use basedrop::Owned;
use std::vec::Drain;

pub(crate) struct Sequences {
	sequence_instances: StaticIndexMap<SequenceInstanceId, Owned<SequenceInstance>>,
	sequence_instances_to_remove: StaticVec<SequenceInstanceId>,
	sequence_output_command_queue: StaticVec<SequenceOutputCommand>,
	output_command_queue: StaticVec<Command>,
}

impl Sequences {
	pub fn new(sequence_capacity: usize, command_capacity: usize) -> Self {
		Self {
			sequence_instances: StaticIndexMap::new(sequence_capacity),
			sequence_instances_to_remove: StaticVec::new(sequence_capacity),
			sequence_output_command_queue: StaticVec::new(command_capacity),
			output_command_queue: StaticVec::new(command_capacity),
		}
	}

	fn start_sequence_instance(
		&mut self,
		id: SequenceInstanceId,
		mut instance: Owned<SequenceInstance>,
	) {
		instance.start();
		self.sequence_instances.try_insert(id, instance).ok();
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
		playables: &Playables,
		metronomes: &Metronomes,
	) -> Drain<Command> {
		// update sequences and process their commands
		for (id, sequence_instance) in &mut self.sequence_instances {
			sequence_instance.update(dt, metronomes, &mut self.sequence_output_command_queue);
			// convert sequence commands to commands that can be consumed
			// by the backend
			for command in self.sequence_output_command_queue.drain(..) {
				match command {
					SequenceOutputCommand::PlaySound(playable_id, instance_id, settings) => {
						if let Some(playable) = playables.playable(playable_id) {
							self.output_command_queue
								.try_push(Command::Instance(InstanceCommand::Play(
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
								)))
								.ok();
						}
					}
					SequenceOutputCommand::SetInstanceVolume(id, volume) => {
						self.output_command_queue
							.try_push(Command::Instance(InstanceCommand::SetInstanceVolume(
								id, volume,
							)))
							.ok();
					}
					SequenceOutputCommand::SetInstancePlaybackRate(id, playback_rate) => {
						self.output_command_queue
							.try_push(Command::Instance(InstanceCommand::SetInstancePlaybackRate(
								id,
								playback_rate,
							)))
							.ok();
					}
					SequenceOutputCommand::SetInstancePanning(id, panning) => {
						self.output_command_queue
							.try_push(Command::Instance(InstanceCommand::SetInstancePanning(
								id, panning,
							)))
							.ok();
					}
					SequenceOutputCommand::PauseInstance(id, settings) => {
						self.output_command_queue
							.try_push(Command::Instance(InstanceCommand::PauseInstance(
								id, settings,
							)))
							.ok();
					}
					SequenceOutputCommand::ResumeInstance(id, settings) => {
						self.output_command_queue
							.try_push(Command::Instance(InstanceCommand::ResumeInstance(
								id, settings,
							)))
							.ok();
					}
					SequenceOutputCommand::StopInstance(id, settings) => {
						self.output_command_queue
							.try_push(Command::Instance(InstanceCommand::StopInstance(
								id, settings,
							)))
							.ok();
					}
					SequenceOutputCommand::PauseInstancesOf(id, settings) => {
						self.output_command_queue
							.try_push(Command::Instance(InstanceCommand::PauseInstancesOf(
								id, settings,
							)))
							.ok();
					}
					SequenceOutputCommand::ResumeInstancesOf(id, settings) => {
						self.output_command_queue
							.try_push(Command::Instance(InstanceCommand::ResumeInstancesOf(
								id, settings,
							)))
							.ok();
					}
					SequenceOutputCommand::StopInstancesOf(id, settings) => {
						self.output_command_queue
							.try_push(Command::Instance(InstanceCommand::StopInstancesOf(
								id, settings,
							)))
							.ok();
					}
					SequenceOutputCommand::PauseSequence(id) => {
						self.output_command_queue
							.try_push(Command::Sequence(SequenceCommand::PauseSequenceInstance(
								id,
							)))
							.ok();
					}
					SequenceOutputCommand::ResumeSequence(id) => {
						self.output_command_queue
							.try_push(Command::Sequence(SequenceCommand::ResumeSequenceInstance(
								id,
							)))
							.ok();
					}
					SequenceOutputCommand::StopSequence(id) => {
						self.output_command_queue
							.try_push(Command::Sequence(SequenceCommand::StopSequenceInstance(id)))
							.ok();
					}
					SequenceOutputCommand::PauseInstancesOfSequence(id, settings) => {
						self.output_command_queue
							.try_push(Command::Instance(
								InstanceCommand::PauseInstancesOfSequence(id, settings),
							))
							.ok();
					}
					SequenceOutputCommand::ResumeInstancesOfSequence(id, settings) => {
						self.output_command_queue
							.try_push(Command::Instance(
								InstanceCommand::ResumeInstancesOfSequence(id, settings),
							))
							.ok();
					}
					SequenceOutputCommand::StopInstancesOfSequence(id, settings) => {
						self.output_command_queue
							.try_push(Command::Instance(InstanceCommand::StopInstancesOfSequence(
								id, settings,
							)))
							.ok();
					}
					SequenceOutputCommand::SetMetronomeTempo(id, tempo) => {
						self.output_command_queue
							.try_push(Command::Metronome(MetronomeCommand::SetMetronomeTempo(
								id, tempo,
							)))
							.ok();
					}
					SequenceOutputCommand::StartMetronome(id) => {
						self.output_command_queue
							.try_push(Command::Metronome(MetronomeCommand::StartMetronome(id)))
							.ok();
					}
					SequenceOutputCommand::PauseMetronome(id) => {
						self.output_command_queue
							.try_push(Command::Metronome(MetronomeCommand::PauseMetronome(id)))
							.ok();
					}
					SequenceOutputCommand::StopMetronome(id) => {
						self.output_command_queue
							.try_push(Command::Metronome(MetronomeCommand::StopMetronome(id)))
							.ok();
					}
					SequenceOutputCommand::SetParameter(id, target, tween) => {
						self.output_command_queue
							.try_push(Command::Parameter(ParameterCommand::SetParameter(
								id, target, tween,
							)))
							.ok();
					}
				}
			}
			if sequence_instance.finished() {
				self.sequence_instances_to_remove.try_push(*id).ok();
			}
		}
		// remove finished sequences
		for id in self.sequence_instances_to_remove.drain(..) {
			self.sequence_instances.remove(&id).unwrap();
		}
		self.output_command_queue.drain(..)
	}
}
