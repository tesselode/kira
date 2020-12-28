use std::{cell::RefCell, rc::Rc, sync::Arc};

use atomic::{Atomic, Ordering};
use indexmap::IndexSet;
use ringbuf::Receiver;

use crate::{
	command::{sender::CommandSender, InstanceCommand, SequenceCommand},
	instance::{PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings},
	AudioError, AudioResult,
};

use super::{SequenceInstanceId, SequenceInstanceState};

pub struct SequenceInstanceHandle<CustomEvent> {
	id: SequenceInstanceId,
	state: Arc<Atomic<SequenceInstanceState>>,
	command_sender: CommandSender,
	raw_event_receiver: Rc<RefCell<Receiver<usize>>>,
	events: IndexSet<CustomEvent>,
}

impl<CustomEvent> SequenceInstanceHandle<CustomEvent> {
	pub(crate) fn new(
		id: SequenceInstanceId,
		state: Arc<Atomic<SequenceInstanceState>>,
		command_sender: CommandSender,
		raw_event_receiver: Receiver<usize>,
		events: IndexSet<CustomEvent>,
	) -> Self {
		Self {
			id,
			state,
			command_sender,
			raw_event_receiver: Rc::new(RefCell::new(raw_event_receiver)),
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
			.push(SequenceCommand::MuteSequenceInstance(self.id).into())
	}

	pub fn unmute(&mut self) -> AudioResult<()> {
		self.command_sender
			.push(SequenceCommand::UnmuteSequenceInstance(self.id).into())
	}

	pub fn pause(&mut self) -> AudioResult<()> {
		self.command_sender
			.push(SequenceCommand::PauseSequenceInstance(self.id).into())
	}

	pub fn resume(&mut self) -> AudioResult<()> {
		self.command_sender
			.push(SequenceCommand::ResumeSequenceInstance(self.id).into())
	}

	pub fn stop(&mut self) -> AudioResult<()> {
		self.command_sender
			.push(SequenceCommand::StopSequenceInstance(self.id).into())
	}

	pub fn pause_sequence_and_instances(
		&mut self,
		settings: PauseInstanceSettings,
	) -> AudioResult<()> {
		self.command_sender
			.push(SequenceCommand::PauseSequenceInstance(self.id).into())?;
		self.command_sender
			.push(InstanceCommand::PauseInstancesOfSequence(self.id, settings).into())?;
		Ok(())
	}

	pub fn resume_sequence_and_instances(
		&mut self,
		settings: ResumeInstanceSettings,
	) -> AudioResult<()> {
		self.command_sender
			.push(SequenceCommand::ResumeSequenceInstance(self.id).into())?;
		self.command_sender
			.push(InstanceCommand::ResumeInstancesOfSequence(self.id, settings).into())?;
		Ok(())
	}

	pub fn stop_sequence_and_instances(
		&mut self,
		settings: StopInstanceSettings,
	) -> AudioResult<()> {
		self.command_sender
			.push(SequenceCommand::StopSequenceInstance(self.id).into())?;
		self.command_sender
			.push(InstanceCommand::StopInstancesOfSequence(self.id, settings).into())?;
		Ok(())
	}

	/// Gets the first event that was emitted since the last
	/// call to `pop_event`.
	pub fn pop_event(&mut self) -> AudioResult<Option<&CustomEvent>> {
		Ok(
			match self
				.raw_event_receiver
				.try_borrow_mut()
				.map_err(|_| AudioError::EventReceiverBorrowed)?
				.pop()
			{
				Some(index) => self.events.get_index(index),
				None => None,
			},
		)
	}
}
