use std::{cell::RefCell, rc::Rc};

use ringbuf::Producer;

use crate::{
	command::{Command, InstanceCommand, ResourceCommand},
	instance::{
		Instance, InstanceHandle, InstanceId, InstanceSettings, PauseInstanceSettings,
		ResumeInstanceSettings, StopInstanceSettings,
	},
	mixer::TrackIndex,
	AudioError, AudioResult,
};

use super::SoundId;

#[derive(Clone)]
pub struct SoundHandle {
	id: SoundId,
	command_producer: Rc<RefCell<Producer<Command>>>,
}

impl SoundHandle {
	pub(crate) fn new(id: SoundId, command_producer: Rc<RefCell<Producer<Command>>>) -> Self {
		Self {
			id,
			command_producer,
		}
	}

	pub fn id(&self) -> SoundId {
		self.id
	}

	pub fn duration(&self) -> f64 {
		self.id.duration()
	}

	pub fn default_track(&self) -> TrackIndex {
		self.id.default_track()
	}

	pub fn semantic_duration(&self) -> Option<f64> {
		self.id.semantic_duration()
	}

	pub fn default_loop_start(&self) -> Option<f64> {
		self.id.default_loop_start()
	}

	fn send_command_to_backend(&mut self, command: Command) -> AudioResult<()> {
		self.command_producer
			.try_borrow_mut()
			.map_err(|_| AudioError::CommandQueueBorrowed)?
			.push(command)
			.map_err(|_| AudioError::CommandQueueFull)
	}

	pub fn play(&mut self, settings: InstanceSettings) -> AudioResult<InstanceHandle> {
		let instance_id = InstanceId::new();
		let instance = Instance::new(self.id.into(), None, settings);
		let handle = InstanceHandle::new(
			instance_id,
			instance.public_state(),
			self.command_producer.clone(),
		);
		self.send_command_to_backend(InstanceCommand::Play(instance_id, instance).into())
			.map(|()| handle)
	}

	pub fn pause(&mut self, settings: PauseInstanceSettings) -> AudioResult<()> {
		self.send_command_to_backend(
			InstanceCommand::PauseInstancesOf(self.id.into(), settings).into(),
		)
	}

	pub fn resume(&mut self, settings: ResumeInstanceSettings) -> AudioResult<()> {
		self.send_command_to_backend(
			InstanceCommand::ResumeInstancesOf(self.id.into(), settings).into(),
		)
	}

	pub fn stop(&mut self, settings: StopInstanceSettings) -> AudioResult<()> {
		self.send_command_to_backend(
			InstanceCommand::StopInstancesOf(self.id.into(), settings).into(),
		)
	}

	pub fn unload(&mut self) -> AudioResult<()> {
		self.stop(Default::default())?;
		self.send_command_to_backend(ResourceCommand::RemoveSound(self.id).into())
	}
}
