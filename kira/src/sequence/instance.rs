use crate::metronome::Metronome;

use super::{Sequence, SequenceOutputCommand, SequenceStep};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SequenceInstanceState {
	Playing,
	Paused,
	Finished,
}

#[derive(Debug, Clone)]
pub struct SequenceInstance<CustomEvent: Copy> {
	sequence: Sequence<CustomEvent>,
	state: SequenceInstanceState,
	position: usize,
	wait_timer: Option<f64>,
	muted: bool,
}

impl<CustomEvent: Copy> SequenceInstance<CustomEvent> {
	pub fn new(sequence: Sequence<CustomEvent>) -> Self {
		Self {
			sequence,
			state: SequenceInstanceState::Playing,
			position: 0,
			wait_timer: None,
			muted: false,
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
							SequenceStep::EmitCustomEvent(event) => {
								todo!();
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
}
