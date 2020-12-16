use std::sync::atomic::{AtomicUsize, Ordering};

use nanorand::{tls_rng, RNG};
use ringbuf::Producer;

use crate::{
	group::{groups::Groups, GroupId},
	metronome::Metronome,
};

use super::{RawSequence, SequenceOutputCommand, SequenceStep};

static NEXT_SEQUENCE_INSTANCE_INDEX: AtomicUsize = AtomicUsize::new(0);

/// A unique identifier for a [`Sequence`](crate::sequence::Sequence).
///
/// You cannot create this manually - a sequence ID is returned
/// when you start a sequence with an [`AudioManager`](crate::manager::AudioManager).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SequenceInstanceId {
	index: usize,
}

impl SequenceInstanceId {
	pub(crate) fn new() -> Self {
		let index = NEXT_SEQUENCE_INSTANCE_INDEX.fetch_add(1, Ordering::Relaxed);
		Self { index }
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SequenceInstanceState {
	Playing,
	Paused,
	Finished,
}

pub struct SequenceInstance {
	sequence: RawSequence,
	state: SequenceInstanceState,
	position: usize,
	wait_timer: Option<f64>,
	muted: bool,
	event_producer: Producer<usize>,
}

impl SequenceInstance {
	pub fn new(sequence: RawSequence, event_producer: Producer<usize>) -> Self {
		Self {
			sequence,
			state: SequenceInstanceState::Playing,
			position: 0,
			wait_timer: None,
			muted: false,
			event_producer,
		}
	}

	fn start_step(&mut self, index: usize) {
		if let Some(step) = self.sequence.steps.get(index) {
			self.position = index;
			if let SequenceStep::Wait(_) = step {
				self.wait_timer = Some(1.0);
			} else {
				self.wait_timer = None;
			}
		} else if let Some(loop_point) = self.sequence.loop_point {
			self.sequence.update_instance_ids();
			self.start_step(loop_point);
		} else {
			self.state = SequenceInstanceState::Finished;
		}
	}

	pub(crate) fn start(&mut self) {
		self.start_step(0);
	}

	pub(crate) fn mute(&mut self) {
		self.muted = true;
	}

	pub(crate) fn unmute(&mut self) {
		self.muted = false;
	}

	pub(crate) fn pause(&mut self) {
		self.state = SequenceInstanceState::Paused;
	}

	pub(crate) fn resume(&mut self) {
		self.state = SequenceInstanceState::Playing;
	}

	pub(crate) fn stop(&mut self) {
		self.state = SequenceInstanceState::Finished;
	}

	pub(crate) fn update(
		&mut self,
		dt: f64,
		metronome: &Metronome,
		output_command_queue: &mut Vec<SequenceOutputCommand>,
	) {
		loop {
			match self.state {
				SequenceInstanceState::Paused | SequenceInstanceState::Finished => {
					break;
				}
				_ => {
					if let Some(step) = self.sequence.steps.get(self.position) {
						match step {
							SequenceStep::Wait(duration) => {
								if let Some(time) = self.wait_timer.as_mut() {
									let duration = duration.in_seconds(metronome.effective_tempo());
									*time -= dt / duration;
									if *time <= 0.0 {
										self.start_step(self.position + 1);
									}
									break;
								}
							}
							SequenceStep::WaitForInterval(interval) => {
								if metronome.interval_passed(*interval) {
									self.start_step(self.position + 1);
								}
								break;
							}
							SequenceStep::RunCommand(command) => {
								if !self.muted {
									output_command_queue.push(*command);
								}
								self.start_step(self.position + 1);
							}
							SequenceStep::PlayRandom(id, choices, settings) => {
								let choice_index = tls_rng().generate_range(0, choices.len());
								if !self.muted {
									output_command_queue.push(SequenceOutputCommand::PlaySound(
										*id,
										choices[choice_index],
										*settings,
									));
								}
								self.start_step(self.position + 1);
							}
							SequenceStep::EmitCustomEvent(event) => {
								if !self.muted {
									match self.event_producer.push(*event) {
										Ok(_) => {}
										Err(_) => {}
									}
								}
								self.start_step(self.position + 1);
							}
						}
					}
				}
			}
		}
	}

	pub(crate) fn finished(&self) -> bool {
		if let SequenceInstanceState::Finished = self.state {
			true
		} else {
			false
		}
	}

	pub(crate) fn is_in_group(&self, parent_id: GroupId, groups: &Groups) -> bool {
		self.sequence.is_in_group(parent_id, groups)
	}
}
