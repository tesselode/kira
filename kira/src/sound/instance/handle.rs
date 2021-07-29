use std::sync::Arc;

use crate::{
	error::CommandError,
	manager::{
		backend::context::Context,
		command::{producer::CommandProducer, Command, InstanceCommand},
	},
	parameter::tween::Tween,
	value::Value,
};

use super::InstanceId;

pub struct InstanceHandle {
	pub(crate) context: Arc<Context>,
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

	pub fn pause(&mut self, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Instance(InstanceCommand::Pause {
				id: self.id,
				tween,
				command_sent_time: self.context.sample_count(),
			}))
	}

	pub fn resume(&mut self, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Instance(InstanceCommand::Resume {
				id: self.id,
				tween,
				command_sent_time: self.context.sample_count(),
			}))
	}

	pub fn stop(&mut self, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Instance(InstanceCommand::Stop {
				id: self.id,
				tween,
				command_sent_time: self.context.sample_count(),
			}))
	}
}
