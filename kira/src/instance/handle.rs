use std::sync::Arc;

use atomic::{Atomic, Ordering};

use crate::{
	command::{producer::CommandProducer, InstanceCommand},
	AudioResult, Value,
};

use super::{
	InstanceId, InstanceState, PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings,
};

pub struct InstanceHandle {
	id: InstanceId,
	state: Arc<Atomic<InstanceState>>,
	command_producer: CommandProducer,
}

impl InstanceHandle {
	pub(crate) fn new(
		id: InstanceId,
		state: Arc<Atomic<InstanceState>>,
		command_producer: CommandProducer,
	) -> Self {
		Self {
			id,
			state,
			command_producer,
		}
	}

	pub fn state(&self) -> InstanceState {
		self.state.load(Ordering::Relaxed)
	}

	pub fn set_volume(&mut self, volume: impl Into<Value<f64>>) -> AudioResult<()> {
		self.command_producer
			.push(InstanceCommand::SetInstanceVolume(self.id, volume.into()).into())
	}

	pub fn set_pitch(&mut self, pitch: impl Into<Value<f64>>) -> AudioResult<()> {
		self.command_producer
			.push(InstanceCommand::SetInstancePitch(self.id, pitch.into()).into())
	}

	pub fn set_panning(&mut self, panning: impl Into<Value<f64>>) -> AudioResult<()> {
		self.command_producer
			.push(InstanceCommand::SetInstancePanning(self.id, panning.into()).into())
	}

	pub fn seek(&mut self, offset: f64) -> AudioResult<()> {
		self.command_producer
			.push(InstanceCommand::SeekInstance(self.id, offset).into())
	}

	pub fn seek_to(&mut self, position: f64) -> AudioResult<()> {
		self.command_producer
			.push(InstanceCommand::SeekInstanceTo(self.id, position).into())
	}

	pub fn pause(&mut self, settings: PauseInstanceSettings) -> AudioResult<()> {
		self.command_producer
			.push(InstanceCommand::PauseInstance(self.id, settings).into())
	}

	pub fn resume(&mut self, settings: ResumeInstanceSettings) -> AudioResult<()> {
		self.command_producer
			.push(InstanceCommand::ResumeInstance(self.id, settings).into())
	}

	pub fn stop(&mut self, settings: StopInstanceSettings) -> AudioResult<()> {
		self.command_producer
			.push(InstanceCommand::StopInstance(self.id, settings).into())
	}
}
