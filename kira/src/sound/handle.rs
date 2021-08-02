use std::sync::Arc;

use atomic_arena::Controller;
use thiserror::Error;

use crate::{error::CommandError, manager::command::{producer::CommandProducer, Command, InstanceCommand}};

use super::{
	data::SoundData,
	instance::{handle::InstanceHandle, settings::InstanceSettings, Instance, InstanceId},
	SoundId, SoundShared,
};

#[derive(Debug, Error)]
pub enum PlaySoundError {
	#[error("Could not add an instance because the maximum number of instances has been reached.")]
	InstanceLimitReached,
	#[error("{0}")]
	CommandError(#[from] CommandError),
}

pub struct SoundHandle {
	pub(crate) id: SoundId,
	pub(crate) data: Arc<dyn SoundData>,
	pub(crate) shared: Arc<SoundShared>,
	pub(crate) instance_controller: Controller,
	pub(crate) command_producer: CommandProducer,
}

impl SoundHandle {
	pub fn id(&self) -> SoundId {
		self.id
	}

	pub fn data(&self) -> &Arc<dyn SoundData> {
		&self.data
	}

	pub fn play(&mut self, settings: InstanceSettings) -> Result<InstanceHandle, PlaySoundError> {
		let id = InstanceId(
			self.instance_controller
				.try_reserve()
				.map_err(|_| PlaySoundError::InstanceLimitReached)?,
		);
		let instance = Instance::new(self.id, &self.data, settings);
		let handle = InstanceHandle {
			id,
			shared: instance.shared(),
			command_producer: self.command_producer.clone(),
		};
		self.command_producer
			.push(Command::Instance(InstanceCommand::Add(id, instance)))?;
		Ok(handle)
	}
}

impl Drop for SoundHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}

impl From<&SoundHandle> for Arc<dyn SoundData> {
	fn from(handle: &SoundHandle) -> Self {
		handle.data.clone()
	}
}
