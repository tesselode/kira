use std::sync::Arc;

use atomic::{Atomic, Ordering};
use flume::Sender;

use crate::{
	command::{Command, InstanceCommand},
	AudioError, AudioResult, Value,
};

use super::{
	InstanceId, InstanceState, PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings,
};

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

	pub fn id(&self) -> InstanceId {
		self.id
	}

	pub fn state(&self) -> InstanceState {
		self.state.load(Ordering::Relaxed)
	}

	pub fn set_volume(&mut self, volume: impl Into<Value<f64>>) -> AudioResult<()> {
		self.command_sender
			.send(InstanceCommand::SetInstanceVolume(self.id, volume.into()).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn set_pitch(&mut self, pitch: impl Into<Value<f64>>) -> AudioResult<()> {
		self.command_sender
			.send(InstanceCommand::SetInstancePitch(self.id, pitch.into()).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn set_panning(&mut self, panning: impl Into<Value<f64>>) -> AudioResult<()> {
		self.command_sender
			.send(InstanceCommand::SetInstancePanning(self.id, panning.into()).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn seek(&mut self, offset: f64) -> AudioResult<()> {
		self.command_sender
			.send(InstanceCommand::SeekInstance(self.id, offset).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn seek_to(&mut self, position: f64) -> AudioResult<()> {
		self.command_sender
			.send(InstanceCommand::SeekInstanceTo(self.id, position).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn pause(&mut self, settings: PauseInstanceSettings) -> AudioResult<()> {
		self.command_sender
			.send(InstanceCommand::PauseInstance(self.id, settings).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn resume(&mut self, settings: ResumeInstanceSettings) -> AudioResult<()> {
		self.command_sender
			.send(InstanceCommand::ResumeInstance(self.id, settings).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}

	pub fn stop(&mut self, settings: StopInstanceSettings) -> AudioResult<()> {
		self.command_sender
			.send(InstanceCommand::StopInstance(self.id, settings).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}
}
