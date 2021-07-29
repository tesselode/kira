use crate::{
	error::CommandError,
	manager::command::{producer::CommandProducer, Command, InstanceCommand},
	value::Value,
};

use super::InstanceId;

pub struct InstanceHandle {
	pub(crate) id: InstanceId,
	pub(crate) command_producer: CommandProducer,
}

impl InstanceHandle {
	pub fn id(&self) -> InstanceId {
		self.id
	}

	pub fn set_volume(&mut self, volume: impl Into<Value>) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Instance(InstanceCommand::SetVolume(
				self.id,
				volume.into(),
			)))
	}

	pub fn set_playback_rate(
		&mut self,
		playback_rate: impl Into<Value>,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Instance(InstanceCommand::SetPlaybackRate(
				self.id,
				playback_rate.into(),
			)))
	}

	pub fn set_panning(&mut self, panning: impl Into<Value>) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Instance(InstanceCommand::SetPanning(
				self.id,
				panning.into(),
			)))
	}
}
