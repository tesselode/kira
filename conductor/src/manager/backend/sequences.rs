use crate::{
	command::SequenceCommand,
	metronome::Metronome,
	sequence::{Sequence, SequenceId, SequenceOutputCommand},
};
use indexmap::IndexMap;
use ringbuf::Producer;
use std::vec::Drain;

pub(crate) struct Sequences<CustomEvent> {
	sequences: IndexMap<SequenceId, Sequence<CustomEvent>>,
	sequences_to_remove: Vec<SequenceId>,
	output_command_queue: Vec<SequenceOutputCommand<CustomEvent>>,
}

impl<CustomEvent: Copy> Sequences<CustomEvent> {
	pub fn new(sequence_capacity: usize, command_capacity: usize) -> Self {
		Self {
			sequences: IndexMap::with_capacity(sequence_capacity),
			sequences_to_remove: Vec::with_capacity(sequence_capacity),
			output_command_queue: Vec::with_capacity(command_capacity),
		}
	}

	pub fn run_command(&mut self, command: SequenceCommand<CustomEvent>) {
		match command {
			SequenceCommand::StartSequence(id, mut sequence) => {
				sequence.start();
				self.sequences.insert(id, sequence);
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
		}
	}

	pub fn update(
		&mut self,
		dt: f32,
		metronome: &Metronome,
		sequences_to_unload_producer: &mut Producer<Sequence<CustomEvent>>,
	) -> Drain<SequenceOutputCommand<CustomEvent>> {
		for (id, sequence) in &mut self.sequences {
			sequence.update(dt, metronome, &mut self.output_command_queue);
			if sequence.finished() {
				self.sequences_to_remove.push(*id);
			}
		}
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
