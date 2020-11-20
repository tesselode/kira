use crate::{
	command::InstanceCommand,
	command::MetronomeCommand,
	command::ParameterCommand,
	command::{Command, SequenceCommand},
	metronome::Metronome,
	sequence::SequenceOutputCommand,
	sequence::{Sequence, SequenceId},
};
use indexmap::IndexMap;
use ringbuf::Producer;
use std::vec::Drain;

pub(crate) struct Sequences<CustomEvent: Copy + std::fmt::Debug> {
	sequences: IndexMap<SequenceId, Sequence<CustomEvent>>,
	sequences_to_remove: Vec<SequenceId>,
	sequence_output_command_queue: Vec<SequenceOutputCommand<CustomEvent>>,
	output_command_queue: Vec<Command<CustomEvent>>,
}

impl<CustomEvent: Copy + std::fmt::Debug> Sequences<CustomEvent> {
	pub fn new(sequence_capacity: usize, command_capacity: usize) -> Self {
		Self {
			sequences: IndexMap::with_capacity(sequence_capacity),
			sequences_to_remove: Vec::with_capacity(sequence_capacity),
			sequence_output_command_queue: Vec::with_capacity(command_capacity),
			output_command_queue: Vec::with_capacity(command_capacity),
		}
	}

	fn start_sequence(&mut self, id: SequenceId, mut sequence: Sequence<CustomEvent>) {
		sequence.start();
		self.sequences.insert(id, sequence);
	}

	pub fn run_command(&mut self, command: SequenceCommand<CustomEvent>) {
		match command {
			SequenceCommand::StartSequence(id, sequence) => {
				self.start_sequence(id, sequence);
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
		sequences_to_unload_producer: &mut Producer<Sequence<CustomEvent>>,
	) -> Drain<Command<CustomEvent>> {
		// update sequences and process their commands
		for (id, sequence) in &mut self.sequences {
			sequence.update(dt, metronome, &mut self.sequence_output_command_queue);
			// convert sequence commands to commands that can be consumed
			// by the backend
			for command in self.sequence_output_command_queue.drain(..) {
				self.output_command_queue.push(match command {
					SequenceOutputCommand::PlaySound(instance_id, sound_id, settings) => {
						Command::Instance(InstanceCommand::PlaySound(
							instance_id,
							sound_id,
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
					SequenceOutputCommand::PauseInstance(id, fade_tween) => {
						Command::Instance(InstanceCommand::PauseInstance(id, fade_tween))
					}
					SequenceOutputCommand::ResumeInstance(id, fade_tween) => {
						Command::Instance(InstanceCommand::ResumeInstance(id, fade_tween))
					}
					SequenceOutputCommand::StopInstance(id, fade_tween) => {
						Command::Instance(InstanceCommand::StopInstance(id, fade_tween))
					}
					SequenceOutputCommand::PauseInstancesOfSound(id, fade_tween) => {
						Command::Instance(InstanceCommand::PauseInstancesOfSound(id, fade_tween))
					}
					SequenceOutputCommand::ResumeInstancesOfSound(id, fade_tween) => {
						Command::Instance(InstanceCommand::ResumeInstancesOfSound(id, fade_tween))
					}
					SequenceOutputCommand::StopInstancesOfSound(id, fade_tween) => {
						Command::Instance(InstanceCommand::StopInstancesOfSound(id, fade_tween))
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
					SequenceOutputCommand::PauseInstancesOfSequence(id, fade_tween) => {
						Command::Instance(InstanceCommand::PauseInstancesOfSequence(id, fade_tween))
					}
					SequenceOutputCommand::ResumeInstancesOfSequence(id, fade_tween) => {
						Command::Instance(InstanceCommand::ResumeInstancesOfSequence(
							id, fade_tween,
						))
					}
					SequenceOutputCommand::StopInstancesOfSequence(id, fade_tween) => {
						Command::Instance(InstanceCommand::StopInstancesOfSequence(id, fade_tween))
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
					SequenceOutputCommand::EmitCustomEvent(event) => {
						Command::EmitCustomEvent(event)
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
