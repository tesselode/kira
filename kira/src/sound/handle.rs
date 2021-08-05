use std::{
	error::Error,
	fmt::{Display, Formatter},
	sync::Arc,
};

use atomic_arena::Controller;

use crate::{
	error::CommandError,
	manager::command::{producer::CommandProducer, Command, InstanceCommand},
};

use super::{
	data::SoundData,
	instance::{Instance, InstanceHandle, InstanceId, InstanceSettings},
	SoundId, SoundShared,
};

/// An error that can occur when playing a sound.
#[derive(Debug)]
pub enum PlaySoundError {
	/// Could not add an instance because the maximum number of instances has been reached.
	InstanceLimitReached,
	/// An error occured when sending a command to the renderer.
	CommandError(CommandError),
}

impl Display for PlaySoundError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
            PlaySoundError::InstanceLimitReached => f.write_str("Could not add an instance because the maximum number of instances has been reached."),
            PlaySoundError::CommandError(error) => error.fmt(f),
        }
	}
}

impl Error for PlaySoundError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			PlaySoundError::CommandError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<CommandError> for PlaySoundError {
	fn from(v: CommandError) -> Self {
		Self::CommandError(v)
	}
}

/// Controls a sound.
///
/// When a [`SoundHandle`] is dropped, the corresponding sound
/// will be removed.
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
