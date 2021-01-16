use flume::Sender;
use thiserror::Error;

use crate::{
	command::{Command, InstanceCommand, SequenceCommand},
	instance::{PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings},
};

use super::GroupId;

#[derive(Debug, Error)]
pub enum GroupHandleError {
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

pub struct GroupHandle {
	id: GroupId,
	command_sender: Sender<Command>,
}

impl GroupHandle {
	pub(crate) fn new(id: GroupId, command_sender: Sender<Command>) -> Self {
		Self { id, command_sender }
	}

	pub fn id(&self) -> GroupId {
		self.id
	}

	/// Pauses all instances of sounds, arrangements, and sequences in this group.
	pub fn pause(&mut self, settings: PauseInstanceSettings) -> Result<(), GroupHandleError> {
		self.command_sender
			.send(InstanceCommand::PauseGroup(self.id().into(), settings).into())
			.map_err(|_| GroupHandleError::BackendDisconnected)?;
		self.command_sender
			.send(SequenceCommand::PauseGroup(self.id().into()).into())
			.map_err(|_| GroupHandleError::BackendDisconnected)?;
		Ok(())
	}

	/// Resumes all instances of sounds, arrangements, and sequences in this group.
	pub fn resume(&mut self, settings: ResumeInstanceSettings) -> Result<(), GroupHandleError> {
		self.command_sender
			.send(InstanceCommand::ResumeGroup(self.id().into(), settings).into())
			.map_err(|_| GroupHandleError::BackendDisconnected)?;
		self.command_sender
			.send(SequenceCommand::ResumeGroup(self.id().into()).into())
			.map_err(|_| GroupHandleError::BackendDisconnected)?;
		Ok(())
	}

	/// Stops all instances of sounds, arrangements, and sequences in this group.
	pub fn stop(&mut self, settings: StopInstanceSettings) -> Result<(), GroupHandleError> {
		self.command_sender
			.send(InstanceCommand::StopGroup(self.id().into(), settings).into())
			.map_err(|_| GroupHandleError::BackendDisconnected)?;
		self.command_sender
			.send(SequenceCommand::StopGroup(self.id().into()).into())
			.map_err(|_| GroupHandleError::BackendDisconnected)?;
		Ok(())
	}
}
