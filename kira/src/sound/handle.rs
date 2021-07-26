use std::sync::{atomic::Ordering, Arc};

use atomic_arena::Controller;

use crate::{
	error::PlaySoundError,
	manager::command::{producer::CommandProducer, Command, InstanceCommand},
};

use super::{
	instance::{Instance, InstanceId},
	SoundId, SoundShared,
};

pub struct SoundHandle {
	pub(crate) id: SoundId,
	pub(crate) shared: Arc<SoundShared>,
	pub(crate) instance_controller: Controller,
	pub(crate) command_producer: CommandProducer,
}

impl SoundHandle {
	pub fn id(&self) -> SoundId {
		self.id
	}

	pub fn play(&mut self) -> Result<(), PlaySoundError> {
		let id = InstanceId(
			self.instance_controller
				.try_reserve()
				.map_err(|_| PlaySoundError::InstanceLimitReached)?,
		);
		let instance = Instance::new(self.id);
		self.command_producer
			.push(Command::Instance(InstanceCommand::Add(id, instance)))?;
		Ok(())
	}
}

impl Drop for SoundHandle {
	fn drop(&mut self) {
		self.shared.removed.store(true, Ordering::SeqCst);
	}
}
