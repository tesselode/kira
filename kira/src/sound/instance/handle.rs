use std::sync::Arc;

use crate::{
	error::InstanceError,
	manager::{
		command::{producer::CommandProducer, Command, InstanceCommand},
		renderer::context::Context,
	},
	parameter::tween::Tween,
	value::Value,
};

use super::{InstanceId, InstanceShared, InstanceState};

pub struct InstanceHandle {
	pub(crate) context: Arc<Context>,
	pub(crate) id: InstanceId,
	pub(crate) shared: Arc<InstanceShared>,
	pub(crate) command_producer: CommandProducer,
}

impl InstanceHandle {
	pub fn id(&self) -> InstanceId {
		self.id
	}

	pub fn state(&self) -> InstanceState {
		self.shared.state()
	}

	pub fn position(&self) -> f64 {
		self.shared.position()
	}

	pub fn set_volume(&mut self, volume: impl Into<Value>) -> Result<(), InstanceError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::SetVolume(
				self.id,
				volume.into(),
			)))?;
		Ok(())
	}

	pub fn set_playback_rate(
		&mut self,
		playback_rate: impl Into<Value>,
	) -> Result<(), InstanceError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::SetPlaybackRate(
				self.id,
				playback_rate.into(),
			)))?;
		Ok(())
	}

	pub fn set_panning(&mut self, panning: impl Into<Value>) -> Result<(), InstanceError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::SetPanning(
				self.id,
				panning.into(),
			)))?;
		Ok(())
	}

	pub fn pause(&mut self, tween: Tween) -> Result<(), InstanceError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::Pause {
				id: self.id,
				tween,
				command_sent_time: self.context.sample_count(),
			}))?;
		Ok(())
	}

	pub fn resume(&mut self, tween: Tween) -> Result<(), InstanceError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::Resume {
				id: self.id,
				tween,
				command_sent_time: self.context.sample_count(),
			}))?;
		Ok(())
	}

	pub fn stop(&mut self, tween: Tween) -> Result<(), InstanceError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::Stop {
				id: self.id,
				tween,
				command_sent_time: self.context.sample_count(),
			}))?;
		Ok(())
	}
}
