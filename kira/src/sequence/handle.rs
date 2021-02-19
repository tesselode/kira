//! An interface for controlling sequence instances.

use std::sync::{Arc, Mutex};

use atomic::{Atomic, Ordering};
use indexmap::IndexSet;
use ringbuf::Consumer;
use thiserror::Error;

use crate::{
	command::{
		producer::{CommandError, CommandProducer},
		InstanceCommand, SequenceCommand,
	},
	instance::{PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings},
};

use super::{SequenceInstanceId, SequenceInstanceState};

/// Something that can go wrong when using a [`SequenceInstanceHandle`]
/// to receive an event from a sequence instance.
#[derive(Debug, Error)]
pub enum PopSequenceInstanceEventError {
	/// A thread panicked while using the event receiver.
	#[error("The event receiver cannot be used because a thread panicked while borrowing it.")]
	MutexPoisoned,
}

/// Allows you to control an instance of a sequence.

// TODO: add a manual impl of Debug
#[derive(Clone)]
pub struct SequenceInstanceHandle<CustomEvent> {
	id: SequenceInstanceId,
	state: Arc<Atomic<SequenceInstanceState>>,
	command_sender: CommandProducer,
	raw_event_receiver: Arc<Mutex<Consumer<usize>>>,
	events: IndexSet<CustomEvent>,
}

impl<CustomEvent> SequenceInstanceHandle<CustomEvent> {
	pub(crate) fn new(
		id: SequenceInstanceId,
		state: Arc<Atomic<SequenceInstanceState>>,
		command_sender: CommandProducer,
		raw_event_receiver: Consumer<usize>,
		events: IndexSet<CustomEvent>,
	) -> Self {
		Self {
			id,
			state,
			command_sender,
			raw_event_receiver: Arc::new(Mutex::new(raw_event_receiver)),
			events,
		}
	}

	/// Returns the ID of the sequence instance.
	pub fn id(&self) -> SequenceInstanceId {
		self.id
	}

	/// Returns the current playback state of the sequence instance.
	pub fn state(&self) -> SequenceInstanceState {
		self.state.load(Ordering::Relaxed)
	}

	/// Mutes the sequence instance.
	///
	/// Muted instances will continue waiting for durations and
	/// intervals, but they will not play sounds, emit events,
	/// or perform any other actions.
	pub fn mute(&mut self) -> Result<(), CommandError> {
		self.command_sender
			.push(SequenceCommand::MuteSequenceInstance(self.id).into())
	}

	/// Unmutes the sequence instance.
	pub fn unmute(&mut self) -> Result<(), CommandError> {
		self.command_sender
			.push(SequenceCommand::UnmuteSequenceInstance(self.id).into())
	}

	/// Pauses the sequence instance.
	pub fn pause(&mut self) -> Result<(), CommandError> {
		self.command_sender
			.push(SequenceCommand::PauseSequenceInstance(self.id).into())
	}

	/// Resumes the sequence instance.
	pub fn resume(&mut self) -> Result<(), CommandError> {
		self.command_sender
			.push(SequenceCommand::ResumeSequenceInstance(self.id).into())
	}

	/// Stops the sequence instance.
	pub fn stop(&mut self) -> Result<(), CommandError> {
		self.command_sender
			.push(SequenceCommand::StopSequenceInstance(self.id).into())
	}

	/// Pauses this sequence instance and all instances of sounds
	/// or arrangements that were started by this sequence instance.
	pub fn pause_sequence_and_instances(
		&mut self,
		settings: PauseInstanceSettings,
	) -> Result<(), CommandError> {
		self.command_sender
			.push(SequenceCommand::PauseSequenceInstance(self.id).into())?;
		self.command_sender
			.push(InstanceCommand::PauseInstancesOfSequence(self.id, settings).into())?;
		Ok(())
	}

	/// Resumes this sequence instance and all instances of sounds
	/// or arrangements that were started by this sequence instance.
	pub fn resume_sequence_and_instances(
		&mut self,
		settings: ResumeInstanceSettings,
	) -> Result<(), CommandError> {
		self.command_sender
			.push(SequenceCommand::ResumeSequenceInstance(self.id).into())?;
		self.command_sender
			.push(InstanceCommand::ResumeInstancesOfSequence(self.id, settings).into())?;
		Ok(())
	}

	/// Stops this sequence instance and all instances of sounds
	/// or arrangements that were started by this sequence instance.
	pub fn stop_sequence_and_instances(
		&mut self,
		settings: StopInstanceSettings,
	) -> Result<(), CommandError> {
		self.command_sender
			.push(SequenceCommand::StopSequenceInstance(self.id).into())?;
		self.command_sender
			.push(InstanceCommand::StopInstancesOfSequence(self.id, settings).into())?;
		Ok(())
	}

	/// Gets the first event that was emitted by this sequence
	/// instance since the last call to `pop_event`.
	pub fn pop_event(&mut self) -> Result<Option<&CustomEvent>, PopSequenceInstanceEventError> {
		let mut raw_event_receiver = self
			.raw_event_receiver
			.lock()
			.map_err(|_| PopSequenceInstanceEventError::MutexPoisoned)?;
		if let Some(index) = raw_event_receiver.pop() {
			Ok(Some(self.events.get_index(index).unwrap()))
		} else {
			Ok(None)
		}
	}
}
