use std::{cell::RefCell, rc::Rc, sync::Arc};

use atomic::{Atomic, Ordering};
use indexmap::IndexSet;
use ringbuf::{Consumer, Producer};

use crate::{
	command::{Command, InstanceCommand, SequenceCommand},
	instance::{PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings},
	AudioError, AudioResult,
};

use super::{SequenceInstanceId, SequenceInstanceState};

pub struct SequenceInstanceHandle<CustomEvent> {
	id: SequenceInstanceId,
	state: Arc<Atomic<SequenceInstanceState>>,
	command_producer: Rc<RefCell<Producer<Command>>>,
	raw_event_consumer: Rc<RefCell<Consumer<usize>>>,
	events: IndexSet<CustomEvent>,
}

impl<CustomEvent> SequenceInstanceHandle<CustomEvent> {
	pub(crate) fn new(
		id: SequenceInstanceId,
		state: Arc<Atomic<SequenceInstanceState>>,
		command_producer: Rc<RefCell<Producer<Command>>>,
		raw_event_consumer: Consumer<usize>,
		events: IndexSet<CustomEvent>,
	) -> Self {
		Self {
			id,
			state,
			command_producer,
			raw_event_consumer: Rc::new(RefCell::new(raw_event_consumer)),
			events,
		}
	}

	pub fn id(&self) -> SequenceInstanceId {
		self.id
	}

	pub fn state(&self) -> SequenceInstanceState {
		self.state.load(Ordering::Relaxed)
	}

	fn send_command_to_backend(&mut self, command: Command) -> AudioResult<()> {
		self.command_producer
			.try_borrow_mut()
			.map_err(|_| AudioError::CommandQueueBorrowed)?
			.push(command)
			.map_err(|_| AudioError::CommandQueueFull)
	}

	pub fn mute(&mut self) -> AudioResult<()> {
		self.send_command_to_backend(SequenceCommand::MuteSequenceInstance(self.id).into())
	}

	pub fn unmute(&mut self) -> AudioResult<()> {
		self.send_command_to_backend(SequenceCommand::UnmuteSequenceInstance(self.id).into())
	}

	pub fn pause(&mut self) -> AudioResult<()> {
		self.send_command_to_backend(SequenceCommand::PauseSequenceInstance(self.id).into())
	}

	pub fn resume(&mut self) -> AudioResult<()> {
		self.send_command_to_backend(SequenceCommand::ResumeSequenceInstance(self.id).into())
	}

	pub fn stop(&mut self) -> AudioResult<()> {
		self.send_command_to_backend(SequenceCommand::StopSequenceInstance(self.id).into())
	}

	pub fn pause_sequence_and_instances(
		&mut self,
		settings: PauseInstanceSettings,
	) -> AudioResult<()> {
		self.send_command_to_backend(SequenceCommand::PauseSequenceInstance(self.id).into())?;
		self.send_command_to_backend(
			InstanceCommand::PauseInstancesOfSequence(self.id, settings).into(),
		)?;
		Ok(())
	}

	pub fn resume_sequence_and_instances(
		&mut self,
		settings: ResumeInstanceSettings,
	) -> AudioResult<()> {
		self.send_command_to_backend(SequenceCommand::ResumeSequenceInstance(self.id).into())?;
		self.send_command_to_backend(
			InstanceCommand::ResumeInstancesOfSequence(self.id, settings).into(),
		)?;
		Ok(())
	}

	pub fn stop_sequence_and_instances(
		&mut self,
		settings: StopInstanceSettings,
	) -> AudioResult<()> {
		self.send_command_to_backend(SequenceCommand::StopSequenceInstance(self.id).into())?;
		self.send_command_to_backend(
			InstanceCommand::StopInstancesOfSequence(self.id, settings).into(),
		)?;
		Ok(())
	}

	/// Gets the first event that was emitted since the last
	/// call to `pop_event`.
	pub fn pop_event(&mut self) -> AudioResult<Option<&CustomEvent>> {
		Ok(
			match self
				.raw_event_consumer
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
