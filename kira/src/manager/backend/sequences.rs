use crate::{
	command::{Command, InstanceCommand, MetronomeCommand, ParameterCommand, SequenceCommand},
	metronome::Metronome,
	sequence::{SequenceInstance, SequenceInstanceId, SequenceOutputCommand},
};
use indexmap::IndexMap;
use ringbuf::Producer;
use std::vec::Drain;

pub(crate) struct Sequences {
	sequences: IndexMap<SequenceInstanceId, SequenceInstance>,
	sequences_to_remove: Vec<SequenceInstanceId>,
	sequence_output_command_queue: Vec<SequenceOutputCommand>,
	output_command_queue: Vec<Command>,
}

impl Sequences {
	pub fn new(sequence_capacity: usize, command_capacity: usize) -> Self {
		Self {
			sequences: IndexMap::with_capacity(sequence_capacity),
			sequences_to_remove: Vec::with_capacity(sequence_capacity),
			sequence_output_command_queue: Vec::with_capacity(command_capacity),
			output_command_queue: Vec::with_capacity(command_capacity),
		}
	}

	fn start_sequence(&mut self, id: SequenceInstanceId, mut instance: SequenceInstance) {
		instance.start();
		self.sequences.insert(id, instance);
	}

	pub fn run_command(&mut self, command: SequenceCommand) {
		match command {
			SequenceCommand::StartSequence(id, instance) => {
				self.start_sequence(id, instance);
			}
			SequenceCommand::MuteSequence(id) => {
				if let Some(sequence) = self.sequences.get_mut(&id) {
					sequence.mute();
				}
			}
			SequenceCommand::UnmuteSequence(id) => {
				if let Some(sequence) = self.sequences.get_mut(&id) {
					sequence.unmute();
				}
			}
			SequenceCommand::PauseSequence(id) => {
				if let Some(sequence) = self.sequences.get_mut(&id) {
					sequence.pause();
				}
			}
			SequenceCommand::ResumeSequence(id) => {
				if let Some(sequence) = self.sequences.get_mut(&id) {
					sequence.resume();
				}
			}
			SequenceCommand::StopSequence(id) => {
				if let Some(sequence) = self.sequences.get_mut(&id) {
					sequence.stop();
				}
			}
		}
	}

	pub fn update(
		&mut self,
		dt: f64,
		metronome: &Metronome,
		sequences_to_unload_producer: &mut Producer<SequenceInstance>,
	) -> Drain<Command> {
		// update sequences and process their commands
		for (id, sequence) in &mut self.sequences {
			sequence.update(dt, metronome, &mut self.sequence_output_command_queue);
			// convert sequence commands to commands that can be consumed
			// by the backend
			for command in self.sequence_output_command_queue.drain(..) {
				self.output_command_queue.push(match command {
					SequenceOutputCommand::PlaySound(instance_id, playable, settings) => {
						Command::Instance(InstanceCommand::Play(
							instance_id,
							playable,
							Some(*id),
							settings,
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
						Command::Sequence(SequenceCommand::PauseSequence(id))
					}
					SequenceOutputCommand::ResumeSequence(id) => {
						Command::Sequence(SequenceCommand::ResumeSequence(id))
					}
					SequenceOutputCommand::StopSequence(id) => {
						Command::Sequence(SequenceCommand::StopSequence(id))
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
			if sequence.finished() {
				self.sequences_to_remove.push(*id);
			}
		}
		// remove finished sequences
		for id in self.sequences_to_remove.drain(..) {
			let sequence = self.sequences.remove(&id).unwrap();
			match sequences_to_unload_producer.push(sequence) {
				Ok(_) => {}
				Err(sequence) => {
					self.sequences.insert(id, sequence);
				}
			}
		}
		self.output_command_queue.drain(..)
	}
}
