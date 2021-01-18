use std::sync::{atomic::Ordering, Arc};

use atomic::Atomic;
use flume::Sender;
use nanorand::{tls_rng, RNG};
use uuid::Uuid;

use crate::{
	group::{groups::Groups, GroupId},
	metronome::{MetronomeId, Metronomes},
	util::generate_uuid,
	Tempo,
};

use super::{RawSequence, SequenceInstanceHandle, SequenceOutputCommand, SequenceStep};

/// A unique identifier for an instance of a [`Sequence`](crate::sequence::Sequence).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
pub struct SequenceInstanceId {
	uuid: Uuid,
}

impl SequenceInstanceId {
	pub(crate) fn new() -> Self {
		Self {
			uuid: generate_uuid(),
		}
	}
}

impl<CustomEvent> From<&SequenceInstanceHandle<CustomEvent>> for SequenceInstanceId {
	fn from(handle: &SequenceInstanceHandle<CustomEvent>) -> Self {
		handle.id()
	}
}

/// The playback state of an instance of a sequence.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SequenceInstanceState {
	/// The sequence instance is playing.
	Playing,
	/// The sequence instance is paused.
	///
	/// Any wait commands are currently on hold.
	Paused,
	/// The sequence has finished and will not perform
	/// any more actions.
	Finished,
}

pub struct SequenceInstance {
	sequence: RawSequence,
	metronome: Option<MetronomeId>,
	state: SequenceInstanceState,
	public_state: Arc<Atomic<SequenceInstanceState>>,
	position: usize,
	wait_timer: Option<f64>,
	muted: bool,
	event_sender: Sender<usize>,
}

impl SequenceInstance {
	pub fn new(
		sequence: RawSequence,
		event_sender: Sender<usize>,
		metronome: Option<MetronomeId>,
	) -> Self {
		Self {
			sequence,
			metronome,
			state: SequenceInstanceState::Playing,
			public_state: Arc::new(Atomic::new(SequenceInstanceState::Playing)),
			position: 0,
			wait_timer: None,
			muted: false,
			event_sender,
		}
	}

	pub fn public_state(&self) -> Arc<Atomic<SequenceInstanceState>> {
		self.public_state.clone()
	}

	fn set_state(&mut self, state: SequenceInstanceState) {
		self.state = state;
		self.public_state.store(state, Ordering::Relaxed);
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
			self.set_state(SequenceInstanceState::Finished);
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
		self.set_state(SequenceInstanceState::Paused);
	}

	pub(crate) fn resume(&mut self) {
		self.set_state(SequenceInstanceState::Playing);
	}

	pub(crate) fn stop(&mut self) {
		self.set_state(SequenceInstanceState::Finished);
	}

	pub(crate) fn update(
		&mut self,
		dt: f64,
		metronomes: &Metronomes,
		output_command_queue: &mut Vec<SequenceOutputCommand>,
	) {
		let metronome = self.metronome.map(|id| metronomes.get(id)).flatten();
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
									let duration =
										duration.in_seconds(if let Some(metronome) = metronome {
											metronome.effective_tempo()
										} else {
											Tempo(0.0)
										});
									*time -= dt / duration;
									if *time <= 0.0 {
										self.start_step(self.position + 1);
									}
									break;
								}
							}
							SequenceStep::WaitForInterval(interval) => {
								if let Some(metronome) = metronome {
									if metronome.interval_passed(*interval) {
										self.start_step(self.position + 1);
									}
								}
								break;
							}
							SequenceStep::RunCommand(command) => {
								if !self.muted {
									output_command_queue.push(*command);
								}
								self.start_step(self.position + 1);
							}
							SequenceStep::PlayRandom(choices, settings) => {
								let choice_index = tls_rng().generate_range(0, choices.len());
								if !self.muted {
									output_command_queue.push(SequenceOutputCommand::PlaySound(
										choices[choice_index],
										*settings,
									));
								}
								self.start_step(self.position + 1);
							}
							SequenceStep::EmitCustomEvent(event) => {
								if !self.muted {
									self.event_sender.try_send(*event).ok();
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
