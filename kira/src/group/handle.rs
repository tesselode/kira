use flume::Sender;

use crate::{
	command::{Command, InstanceCommand, SequenceCommand},
	instance::{PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings},
	AudioError, AudioResult,
};

use super::GroupId;

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
	pub fn pause(&mut self, settings: PauseInstanceSettings) -> AudioResult<()> {
		self.command_sender
			.send(InstanceCommand::PauseGroup(self.id().into(), settings).into())
			.map_err(|_| AudioError::BackendDisconnected)?;
		self.command_sender
			.send(SequenceCommand::PauseGroup(self.id().into()).into())
			.map_err(|_| AudioError::BackendDisconnected)?;
		Ok(())
	}

	/// Resumes all instances of sounds, arrangements, and sequences in this group.
	pub fn resume(&mut self, settings: ResumeInstanceSettings) -> AudioResult<()> {
		self.command_sender
			.send(InstanceCommand::ResumeGroup(self.id().into(), settings).into())
			.map_err(|_| AudioError::BackendDisconnected)?;
		self.command_sender
			.send(SequenceCommand::ResumeGroup(self.id().into()).into())
			.map_err(|_| AudioError::BackendDisconnected)?;
		Ok(())
	}

	/// Stops all instances of sounds, arrangements, and sequences in this group.
	pub fn stop(&mut self, settings: StopInstanceSettings) -> AudioResult<()> {
		self.command_sender
			.send(InstanceCommand::StopGroup(self.id().into(), settings).into())
			.map_err(|_| AudioError::BackendDisconnected)?;
		self.command_sender
			.send(SequenceCommand::StopGroup(self.id().into()).into())
			.map_err(|_| AudioError::BackendDisconnected)?;
		Ok(())
	}
}
