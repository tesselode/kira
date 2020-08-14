use crate::{
	command::SequenceCommand,
	metronome::Metronome,
	sequence::{Sequence, SequenceId, SequenceOutputCommand},
};
use indexmap::IndexMap;
use std::vec::Drain;

pub(crate) struct Sequences {
	sequences: IndexMap<SequenceId, Sequence>,
	sequences_to_remove: Vec<SequenceId>,
	output_command_queue: Vec<SequenceOutputCommand>,
}

impl Sequences {
	pub fn new(sequence_capacity: usize, command_capacity: usize) -> Self {
		Self {
			sequences: IndexMap::with_capacity(sequence_capacity),
			sequences_to_remove: Vec::with_capacity(sequence_capacity),
			output_command_queue: Vec::with_capacity(command_capacity),
		}
	}

	pub fn run_command(&mut self, command: SequenceCommand) {
		match command {
			SequenceCommand::StartSequence(id, mut sequence) => {
				sequence.start();
				self.sequences.insert(id, sequence);
			}
		}
	}

	pub fn update(&mut self, dt: f32, metronome: &Metronome) -> Drain<SequenceOutputCommand> {
		for (id, sequence) in &mut self.sequences {
			sequence.update(dt, metronome, &mut self.output_command_queue);
			if sequence.finished() {
				self.sequences_to_remove.push(*id);
			}
		}
		for id in self.sequences_to_remove.drain(..) {
			self.sequences.remove(&id);
		}
		self.output_command_queue.drain(..)
	}
}
