use std::sync::Arc;

use atomic::{Atomic, Ordering};
use flume::{Receiver, Sender};
use indexmap::IndexSet;

use crate::{
	command::{Command, InstanceCommand, SequenceCommand},
	instance::{PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings},
	AudioError, AudioResult,
};

use super::{SequenceInstanceId, SequenceInstanceState};

#[derive(Clone)]
pub struct SequenceInstanceHandle<CustomEvent> {
	id: SequenceInstanceId,
	state: Arc<Atomic<SequenceInstanceState>>,
	command_sender: Sender<Command>,
	raw_event_receiver: Receiver<usize>,
	events: IndexSet<CustomEvent>,
}

impl<CustomEvent> SequenceInstanceHandle<CustomEvent> {
	pub(crate) fn new(
		id: SequenceInstanceId,
		state: Arc<Atomic<SequenceInstanceState>>,
		command_sender: Sender<Command>,
		raw_event_receiver: Receiver<usize>,
		events: IndexSet<CustomEvent>,
	) -> Self {
		Self {
			id,
			state,
			command_sender,
			raw_event_receiver,
			events,
		}
	}

	pub fn id(&self) -> SequenceInstanceId {
		self.id
	}

	pub fn state(&self) -> SequenceInstanceState {
		self.state.load(Ordering::Relaxed)
	}

	pub fn mute(&mut self) -> AudioResult<()> {
		self.command_sender
			.send(SequenceCommand::MuteSequenceInstance(self.id).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn unmute(&mut self) -> AudioResult<()> {
		self.command_sender
			.send(SequenceCommand::UnmuteSequenceInstance(self.id).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn pause(&mut self) -> AudioResult<()> {
		self.command_sender
			.send(SequenceCommand::PauseSequenceInstance(self.id).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn resume(&mut self) -> AudioResult<()> {
		self.command_sender
			.send(SequenceCommand::ResumeSequenceInstance(self.id).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn stop(&mut self) -> AudioResult<()> {
		self.command_sender
			.send(SequenceCommand::StopSequenceInstance(self.id).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn pause_sequence_and_instances(
		&mut self,
		settings: PauseInstanceSettings,
	) -> AudioResult<()> {
		self.command_sender
			.send(SequenceCommand::PauseSequenceInstance(self.id).into())
			.map_err(|_| AudioError::BackendDisconnected)?;
		self.command_sender
			.send(InstanceCommand::PauseInstancesOfSequence(self.id, settings).into())
			.map_err(|_| AudioError::BackendDisconnected)?;
		Ok(())
	}

	pub fn resume_sequence_and_instances(
		&mut self,
		settings: ResumeInstanceSettings,
	) -> AudioResult<()> {
		self.command_sender
			.send(SequenceCommand::ResumeSequenceInstance(self.id).into())
			.map_err(|_| AudioError::BackendDisconnected)?;
		self.command_sender
			.send(InstanceCommand::ResumeInstancesOfSequence(self.id, settings).into())
			.map_err(|_| AudioError::BackendDisconnected)?;
		Ok(())
	}

	pub fn stop_sequence_and_instances(
		&mut self,
		settings: StopInstanceSettings,
	) -> AudioResult<()> {
		self.command_sender
			.send(SequenceCommand::StopSequenceInstance(self.id).into())
			.map_err(|_| AudioError::BackendDisconnected)?;
		self.command_sender
			.send(InstanceCommand::StopInstancesOfSequence(self.id, settings).into())
			.map_err(|_| AudioError::BackendDisconnected)?;
		Ok(())
	}

	/// Gets the first event that was emitted since the last
	/// call to `pop_event`.
	pub fn pop_event(&mut self) -> AudioResult<Option<&CustomEvent>> {
		Ok(match self.raw_event_receiver.try_recv().ok() {
			Some(index) => Some(self.events.get_index(index).unwrap()),
			None => None,
		})
	}
}
