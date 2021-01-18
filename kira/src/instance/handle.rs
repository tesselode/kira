//! An interface for controlling instances of sounds and arrangements.

use std::sync::Arc;

use atomic::{Atomic, Ordering};
use flume::Sender;
use thiserror::Error;

use crate::{
	command::{Command, InstanceCommand},
	Value,
};

use super::{
	InstanceId, InstanceState, PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings,
};

/// Something that can go wrong when using an [`InstanceHandle`] to
/// control a instance.
#[derive(Debug, Error)]
pub enum InstanceHandleError {
	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Allows you to control an instance of a sound or arrangement.
pub struct InstanceHandle {
	id: InstanceId,
	state: Arc<Atomic<InstanceState>>,
	command_sender: Sender<Command>,
}

impl InstanceHandle {
	pub(crate) fn new(
		id: InstanceId,
		state: Arc<Atomic<InstanceState>>,
		command_sender: Sender<Command>,
	) -> Self {
		Self {
			id,
			state,
			command_sender,
		}
	}

	/// Returns the ID of the instance.
	pub fn id(&self) -> InstanceId {
		self.id
	}

	/// Returns the playback state of the instance.
	pub fn state(&self) -> InstanceState {
		self.state.load(Ordering::Relaxed)
	}

	/// Sets the volume of the instance.
	pub fn set_volume(&mut self, volume: impl Into<Value<f64>>) -> Result<(), InstanceHandleError> {
		self.command_sender
			.send(InstanceCommand::SetInstanceVolume(self.id, volume.into()).into())
			.map_err(|_| InstanceHandleError::BackendDisconnected)
	}

	/// Sets the pitch of the instance.
	pub fn set_pitch(&mut self, pitch: impl Into<Value<f64>>) -> Result<(), InstanceHandleError> {
		self.command_sender
			.send(InstanceCommand::SetInstancePitch(self.id, pitch.into()).into())
			.map_err(|_| InstanceHandleError::BackendDisconnected)
	}

	/// Sets the panning of the instance.
	pub fn set_panning(
		&mut self,
		panning: impl Into<Value<f64>>,
	) -> Result<(), InstanceHandleError> {
		self.command_sender
			.send(InstanceCommand::SetInstancePanning(self.id, panning.into()).into())
			.map_err(|_| InstanceHandleError::BackendDisconnected)
	}

	/// Offsets the playback position of the instance by the specified amount (in seconds).
	pub fn seek(&mut self, offset: f64) -> Result<(), InstanceHandleError> {
		self.command_sender
			.send(InstanceCommand::SeekInstance(self.id, offset).into())
			.map_err(|_| InstanceHandleError::BackendDisconnected)
	}

	/// Sets the playback position of the instance to the specified time (in seconds).
	pub fn seek_to(&mut self, position: f64) -> Result<(), InstanceHandleError> {
		self.command_sender
			.send(InstanceCommand::SeekInstanceTo(self.id, position).into())
			.map_err(|_| InstanceHandleError::BackendDisconnected)
	}

	/// Pauses the instance.
	pub fn pause(&mut self, settings: PauseInstanceSettings) -> Result<(), InstanceHandleError> {
		self.command_sender
			.send(InstanceCommand::PauseInstance(self.id, settings).into())
			.map_err(|_| InstanceHandleError::BackendDisconnected)
	}

	/// Resumes the instance.
	pub fn resume(&mut self, settings: ResumeInstanceSettings) -> Result<(), InstanceHandleError> {
		self.command_sender
			.send(InstanceCommand::ResumeInstance(self.id, settings).into())
			.map_err(|_| InstanceHandleError::BackendDisconnected)
	}

	/// Stops the instance.
	pub fn stop(&mut self, settings: StopInstanceSettings) -> Result<(), InstanceHandleError> {
		self.command_sender
			.send(InstanceCommand::StopInstance(self.id, settings).into())
			.map_err(|_| InstanceHandleError::BackendDisconnected)
	}
}
