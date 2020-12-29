use crate::{
	command::{sender::CommandSender, InstanceCommand, SequenceCommand},
	instance::{PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings},
	AudioResult,
};

use super::GroupId;

pub struct GroupHandle {
	id: GroupId,
	command_sender: CommandSender,
}

impl GroupHandle {
	pub(crate) fn new(id: GroupId, command_sender: CommandSender) -> Self {
		Self { id, command_sender }
	}

	pub fn id(&self) -> GroupId {
		self.id
	}

	/// Pauses all instances of sounds, arrangements, and sequences in this group.
	pub fn pause(&mut self, settings: PauseInstanceSettings) -> AudioResult<()> {
		self.command_sender
			.push(InstanceCommand::PauseGroup(self.id().into(), settings).into())?;
		self.command_sender
			.push(SequenceCommand::PauseGroup(self.id().into()).into())?;
		Ok(())
	}

	/// Resumes all instances of sounds, arrangements, and sequences in this group.
	pub fn resume(&mut self, settings: ResumeInstanceSettings) -> AudioResult<()> {
		self.command_sender
			.push(InstanceCommand::ResumeGroup(self.id().into(), settings).into())?;
		self.command_sender
			.push(SequenceCommand::ResumeGroup(self.id().into()).into())?;
		Ok(())
	}

	/// Stops all instances of sounds, arrangements, and sequences in this group.
	pub fn stop(&mut self, settings: StopInstanceSettings) -> AudioResult<()> {
		self.command_sender
			.push(InstanceCommand::StopGroup(self.id().into(), settings).into())?;
		self.command_sender
			.push(SequenceCommand::StopGroup(self.id().into()).into())?;
		Ok(())
	}
}
