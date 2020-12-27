use std::{cell::RefCell, rc::Rc, sync::Arc};

use atomic::{Atomic, Ordering};
use ringbuf::Producer;

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
	command_producer: Rc<RefCell<Producer<Command>>>,
}

impl InstanceHandle {
	pub(crate) fn new(
		id: InstanceId,
		state: Arc<Atomic<InstanceState>>,
		command_producer: Rc<RefCell<Producer<Command>>>,
	) -> Self {
		Self {
			id,
			state,
			command_producer,
		}
	}

	fn send_command_to_backend(&mut self, command: Command) -> AudioResult<()> {
		self.command_producer
			.try_borrow_mut()
			.map_err(|_| AudioError::CommandQueueBorrowed)?
			.push(command)
			.map_err(|_| AudioError::CommandQueueFull)
	}

	pub fn state(&self) -> InstanceState {
		self.state.load(Ordering::Relaxed)
	}

	pub fn set_volume(&mut self, volume: impl Into<Value<f64>>) -> AudioResult<()> {
		self.send_command_to_backend(
			InstanceCommand::SetInstanceVolume(self.id, volume.into()).into(),
		)
	}

	pub fn set_pitch(&mut self, pitch: impl Into<Value<f64>>) -> AudioResult<()> {
		self.send_command_to_backend(
			InstanceCommand::SetInstancePitch(self.id, pitch.into()).into(),
		)
	}

	pub fn set_panning(&mut self, panning: impl Into<Value<f64>>) -> AudioResult<()> {
		self.send_command_to_backend(
			InstanceCommand::SetInstancePanning(self.id, panning.into()).into(),
		)
	}

	pub fn seek(&mut self, offset: f64) -> AudioResult<()> {
		self.send_command_to_backend(InstanceCommand::SeekInstance(self.id, offset).into())
	}

	pub fn seek_to(&mut self, position: f64) -> AudioResult<()> {
		self.send_command_to_backend(InstanceCommand::SeekInstanceTo(self.id, position).into())
	}

	pub fn pause(&mut self, settings: PauseInstanceSettings) -> AudioResult<()> {
		self.send_command_to_backend(InstanceCommand::PauseInstance(self.id, settings).into())
	}

	pub fn resume(&mut self, settings: ResumeInstanceSettings) -> AudioResult<()> {
		self.send_command_to_backend(InstanceCommand::ResumeInstance(self.id, settings).into())
	}

	pub fn stop(&mut self, settings: StopInstanceSettings) -> AudioResult<()> {
		self.send_command_to_backend(InstanceCommand::StopInstance(self.id, settings).into())
	}
}
