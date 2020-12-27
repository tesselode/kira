use std::{cell::RefCell, rc::Rc};

use ringbuf::Producer;

use crate::{
	command::{Command, InstanceCommand, ResourceCommand},
	instance::{
		InstanceHandle, InstanceId, InstanceSettings, PauseInstanceSettings,
		ResumeInstanceSettings, StopInstanceSettings,
	},
	AudioError, AudioResult,
};

use super::ArrangementId;

#[derive(Clone)]
pub struct ArrangementHandle {
	id: ArrangementId,
	command_producer: Rc<RefCell<Producer<Command>>>,
}

impl ArrangementHandle {
	pub(crate) fn new(id: ArrangementId, command_producer: Rc<RefCell<Producer<Command>>>) -> Self {
		Self {
			id,
			command_producer,
		}
	}

	pub fn id(&self) -> ArrangementId {
		self.id
	}

	fn send_command_to_backend(&mut self, command: Command) -> AudioResult<()> {
		self.command_producer
			.try_borrow_mut()
			.map_err(|_| AudioError::CommandQueueBorrowed)?
			.push(command)
			.map_err(|_| AudioError::CommandQueueFull)
	}

	pub fn play(&mut self, settings: InstanceSettings) -> AudioResult<InstanceHandle> {
		let instance_id = InstanceId::new();
		self.send_command_to_backend(
			InstanceCommand::Play(instance_id, self.id.into(), None, settings).into(),
		)
		.map(|()| InstanceHandle::new(instance_id, self.command_producer.clone()))
	}

	pub fn pause(&mut self, settings: PauseInstanceSettings) -> AudioResult<()> {
		self.send_command_to_backend(
			InstanceCommand::PauseInstancesOf(self.id.into(), settings).into(),
		)
	}

	pub fn resume(&mut self, settings: ResumeInstanceSettings) -> AudioResult<()> {
		self.send_command_to_backend(
			InstanceCommand::ResumeInstancesOf(self.id.into(), settings).into(),
		)
	}

	pub fn stop(&mut self, settings: StopInstanceSettings) -> AudioResult<()> {
		self.send_command_to_backend(
			InstanceCommand::StopInstancesOf(self.id.into(), settings).into(),
		)
	}

	pub fn unload(&mut self) -> AudioResult<()> {
		self.stop(Default::default())?;
		self.send_command_to_backend(ResourceCommand::RemoveArrangement(self.id).into())
	}
}
