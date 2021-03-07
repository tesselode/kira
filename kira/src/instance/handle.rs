//! An interface for controlling instances of sounds and arrangements.

use std::sync::Arc;

use atomic::{Atomic, Ordering};

use crate::{
	command::{
		producer::{CommandError, CommandProducer},
		InstanceCommand,
	},
	Value,
};

use super::{
	InstanceId, InstanceState, PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings,
};

#[derive(Debug, Clone)]
/// Allows you to control an instance of a sound or arrangement.
pub struct InstanceHandle {
	id: InstanceId,
	state: Arc<Atomic<InstanceState>>,
	position: Arc<Atomic<f64>>,
	command_producer: CommandProducer,
}

impl InstanceHandle {
	pub(crate) fn new(
		id: InstanceId,
		state: Arc<Atomic<InstanceState>>,
		position: Arc<Atomic<f64>>,
		command_producer: CommandProducer,
	) -> Self {
		Self {
			id,
			state,
			position,
			command_producer,
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

	/// Returns the playback position of the instance.
	pub fn position(&self) -> f64 {
		self.position.load(Ordering::Relaxed)
	}

	/// Sets the volume of the instance.
	pub fn set_volume(&mut self, volume: impl Into<Value<f64>>) -> Result<(), CommandError> {
		self.command_producer
			.push(InstanceCommand::SetInstanceVolume(self.id, volume.into()).into())
	}

	/// Sets the playback rate of the instance.
	pub fn set_playback_rate(
		&mut self,
		playback_rate: impl Into<Value<f64>>,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(InstanceCommand::SetInstancePlaybackRate(self.id, playback_rate.into()).into())
	}

	/// Sets the panning of the instance.
	pub fn set_panning(&mut self, panning: impl Into<Value<f64>>) -> Result<(), CommandError> {
		self.command_producer
			.push(InstanceCommand::SetInstancePanning(self.id, panning.into()).into())
	}

	/// Offsets the playback position of the instance by the specified amount (in seconds).
	pub fn seek(&mut self, offset: f64) -> Result<(), CommandError> {
		self.command_producer
			.push(InstanceCommand::SeekInstance(self.id, offset).into())
	}

	/// Sets the playback position of the instance to the specified time (in seconds).
	pub fn seek_to(&mut self, position: f64) -> Result<(), CommandError> {
		self.command_producer
			.push(InstanceCommand::SeekInstanceTo(self.id, position).into())
	}

	/// Pauses the instance.
	pub fn pause(&mut self, settings: PauseInstanceSettings) -> Result<(), CommandError> {
		self.command_producer
			.push(InstanceCommand::PauseInstance(self.id, settings).into())
	}

	/// Resumes the instance.
	pub fn resume(&mut self, settings: ResumeInstanceSettings) -> Result<(), CommandError> {
		self.command_producer
			.push(InstanceCommand::ResumeInstance(self.id, settings).into())
	}

	/// Stops the instance.
	pub fn stop(&mut self, settings: StopInstanceSettings) -> Result<(), CommandError> {
		self.command_producer
			.push(InstanceCommand::StopInstance(self.id, settings).into())
	}
}
