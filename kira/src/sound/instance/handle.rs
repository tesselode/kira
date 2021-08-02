use std::sync::Arc;

use crate::{
	error::CommandError,
	manager::command::{producer::CommandProducer, Command, InstanceCommand},
	parameter::Tween,
	value::Value,
};

use thiserror::Error;

use super::{InstanceId, InstanceShared, InstanceState};

#[derive(Debug, Error)]
pub enum InstanceHandleError {
	#[error("Cannot modify an instance that has finished playing")]
	InstanceStopped,
	#[error("{0}")]
	CommandError(#[from] CommandError),
}

pub struct InstanceHandle {
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

	pub fn set_volume(&mut self, volume: impl Into<Value>) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
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
	) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::SetPlaybackRate(
				self.id,
				playback_rate.into(),
			)))?;
		Ok(())
	}

	pub fn set_panning(&mut self, panning: impl Into<Value>) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::SetPanning(
				self.id,
				panning.into(),
			)))?;
		Ok(())
	}

	pub fn pause(&mut self, fade_out_tween: Tween) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::Pause {
				id: self.id,
				tween: fade_out_tween,
			}))?;
		Ok(())
	}

	pub fn resume(&mut self, fade_in_tween: Tween) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::Resume {
				id: self.id,
				tween: fade_in_tween,
			}))?;
		Ok(())
	}

	pub fn stop(&mut self, fade_out_tween: Tween) -> Result<(), InstanceHandleError> {
		if self.state() == InstanceState::Stopped {
			return Err(InstanceHandleError::InstanceStopped);
		}
		self.command_producer
			.push(Command::Instance(InstanceCommand::Stop {
				id: self.id,
				tween: fade_out_tween,
			}))?;
		Ok(())
	}
}
