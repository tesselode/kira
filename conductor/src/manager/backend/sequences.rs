use crate::{
	command::SequenceCommand,
	duration::Duration,
	instance::InstanceSettings,
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

	fn start_sequence(&mut self, id: SequenceId, mut sequence: Sequence<CustomEvent>) {
		sequence.start();
		self.sequences.insert(id, sequence);
	}

	pub fn run_command(&mut self, command: SequenceCommand<CustomEvent>, metronome: &Metronome) {
		match command {
			SequenceCommand::StartSequence(id, sequence) => {
				self.start_sequence(id, sequence);
			}
			SequenceCommand::LoopSound(id, sound_id, loop_settings, instance_settings) => {
				let tempo = sound_id
					.metadata()
					.tempo
					.unwrap_or(metronome.settings.tempo);
				let duration = sound_id
					.metadata()
					.semantic_duration
					.unwrap_or(Duration::Seconds(sound_id.duration()));
				let start = loop_settings
					.start
					.unwrap_or(Duration::Seconds(0.0))
					.in_seconds(tempo);
				let end = loop_settings.end.unwrap_or(duration).in_seconds(tempo);
				let mut sequence = Sequence::new();
				sequence.play_sound(sound_id, instance_settings);
				sequence.wait(Duration::Seconds(end - instance_settings.position));
				sequence.play_sound(
					sound_id,
					InstanceSettings {
						position: start,
						..instance_settings
					},
				);
				sequence.wait(Duration::Seconds(end - start));
				sequence.go_to(2);
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
		}
	}

	pub fn update(
		&mut self,
		dt: f64,
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
