pub mod error;
pub(crate) mod instance;

use std::sync::Arc;

use crate::{sound::Sound, Duration};

use self::error::SequenceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceLocalInstanceId(usize);

enum SequenceStep<Event> {
	Wait(Duration),
	WaitForInterval(f64),
	PlaySound(SequenceLocalInstanceId, Arc<Sound>),
	PauseInstance(SequenceLocalInstanceId),
	ResumeInstance(SequenceLocalInstanceId),
	StopInstance(SequenceLocalInstanceId),
	Emit(Event),
}

pub struct Sequence<Event> {
	steps: Vec<SequenceStep<Event>>,
	loop_point: Option<usize>,
	next_instance_id: usize,
}

impl<Event> Sequence<Event> {
	pub fn new() -> Self {
		Self {
			steps: vec![],
			loop_point: None,
			next_instance_id: 0,
		}
	}

	/// Adds a step to wait for a certain length of time
	/// before moving to the next step.
	pub fn wait(&mut self, duration: Duration) {
		self.steps.push(SequenceStep::Wait(duration));
	}

	/// Adds a step to wait for a certain metronome interval
	/// (in beats) to be passed before moving to the next step.
	pub fn wait_for_interval(&mut self, interval: f64) {
		self.steps.push(SequenceStep::WaitForInterval(interval));
	}

	/// Marks the point the sequence will loop back to
	/// after it finishes the last step.
	pub fn start_loop(&mut self) {
		self.loop_point = Some(self.steps.len())
	}

	pub fn play(&mut self, sound: Arc<Sound>) -> SequenceLocalInstanceId {
		let id = SequenceLocalInstanceId(self.next_instance_id);
		self.next_instance_id += 1;
		self.steps.push(SequenceStep::PlaySound(id, sound));
		id
	}

	pub fn pause_instance(&mut self, instance: SequenceLocalInstanceId) {
		self.steps.push(SequenceStep::PauseInstance(instance));
	}

	pub fn resume_instance(&mut self, instance: SequenceLocalInstanceId) {
		self.steps.push(SequenceStep::ResumeInstance(instance));
	}

	pub fn stop_instance(&mut self, instance: SequenceLocalInstanceId) {
		self.steps.push(SequenceStep::StopInstance(instance));
	}

	pub fn emit(&mut self, event: Event) {
		self.steps.push(SequenceStep::Emit(event));
	}

	/// Makes sure nothing's wrong with the sequence that would make
	/// it unplayable.
	///
	/// Currently, this only checks that the looping portion of a
	/// sequence (if there is one) contains at least one wait command
	/// (to prevent infinite loops).
	pub(crate) fn validate(&self) -> Result<(), SequenceError> {
		if let Some(loop_point) = self.loop_point {
			for step in self.steps.iter().skip(loop_point) {
				match step {
					SequenceStep::Wait(_) | SequenceStep::WaitForInterval(_) => {
						return Ok(());
					}
					_ => {}
				}
			}
			Err(SequenceError::InfiniteLoop)
		} else {
			Ok(())
		}
	}

	/// Returns the number of instances this sequence will create
	/// in one loop. This is used by sequence instances to determine
	/// how many `InstanceController`s need to be allocated.
	pub(crate) fn num_instances(&self) -> usize {
		self.steps
			.iter()
			.filter(|step| {
				if let SequenceStep::PlaySound(..) = step {
					true
				} else {
					false
				}
			})
			.count()
	}
}

type RawSequence = Sequence<usize>;
