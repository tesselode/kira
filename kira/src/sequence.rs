pub mod error;
pub(crate) mod instance;

use std::{
	collections::{HashMap, HashSet},
	hash::Hash,
};

use basedrop::Shared;

use crate::{
	sound::{handle::SoundHandle, instance::settings::InstanceSettings, Sound},
	Duration,
};

use self::error::SequenceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceLocalInstanceId(usize);

enum SequenceStep<Event: Clone + Eq + Hash> {
	Wait(Duration),
	WaitForInterval(f64),
	PlaySound {
		id: SequenceLocalInstanceId,
		sound: Shared<Sound>,
		settings: InstanceSettings,
	},
	PauseInstance(SequenceLocalInstanceId),
	ResumeInstance(SequenceLocalInstanceId),
	StopInstance(SequenceLocalInstanceId),
	Emit(Event),
}

pub struct Sequence<Event: Clone + Eq + Hash> {
	steps: Vec<SequenceStep<Event>>,
	loop_point: Option<usize>,
	next_instance_id: usize,
}

impl<Event: Clone + Eq + Hash> Sequence<Event> {
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

	pub fn play(
		&mut self,
		sound: &SoundHandle,
		settings: InstanceSettings,
	) -> SequenceLocalInstanceId {
		let id = SequenceLocalInstanceId(self.next_instance_id);
		self.next_instance_id += 1;
		self.steps.push(SequenceStep::PlaySound {
			id,
			sound: sound.sound().clone(),
			settings,
		});
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
				if let SequenceStep::PlaySound { .. } = step {
					true
				} else {
					false
				}
			})
			.count()
	}

	pub(crate) fn create_raw_sequence(&self) -> (RawSequence, Vec<Event>) {
		let mut events = HashSet::new();
		for step in &self.steps {
			if let SequenceStep::Emit(event) = step {
				events.insert(event.clone());
			}
		}
		let events: Vec<Event> = events.drain().collect();
		let event_indices: HashMap<Event, usize> = events
			.iter()
			.enumerate()
			.map(|(i, event)| (event.clone(), i))
			.collect();
		let raw_steps: Vec<SequenceStep<usize>> = self
			.steps
			.iter()
			.map(|step| match step {
				SequenceStep::Wait(duration) => SequenceStep::Wait(*duration),
				SequenceStep::WaitForInterval(interval) => SequenceStep::WaitForInterval(*interval),
				SequenceStep::PlaySound {
					id,
					sound,
					settings,
				} => SequenceStep::PlaySound {
					id: *id,
					sound: sound.clone(),
					settings: settings.clone(),
				},
				SequenceStep::PauseInstance(id) => SequenceStep::PauseInstance(*id),
				SequenceStep::ResumeInstance(id) => SequenceStep::ResumeInstance(*id),
				SequenceStep::StopInstance(id) => SequenceStep::StopInstance(*id),
				SequenceStep::Emit(event) => SequenceStep::Emit(event_indices[event]),
			})
			.collect();
		let raw_sequence = Sequence {
			steps: raw_steps,
			loop_point: self.loop_point,
			next_instance_id: self.next_instance_id,
		};
		(raw_sequence, events)
	}
}

type RawSequence = Sequence<usize>;
