use crate::{
	command::{producer::CommandProducer, ParameterCommand},
	AudioResult,
};

use super::{ParameterId, Tween};

pub struct ParameterHandle {
	id: ParameterId,
	command_producer: CommandProducer,
}

impl ParameterHandle {
	pub(crate) fn new(id: ParameterId, command_producer: CommandProducer) -> Self {
		Self {
			id,
			command_producer,
		}
	}

	pub fn id(&self) -> ParameterId {
		self.id
	}

	pub fn remove(&mut self) -> AudioResult<()> {
		self.command_producer
			.push(ParameterCommand::RemoveParameter(self.id).into())
	}

	pub fn set(&mut self, value: f64, tween: impl Into<Option<Tween>>) -> AudioResult<()> {
		self.command_producer
			.push(ParameterCommand::SetParameter(self.id, value, tween.into()).into())
	}
}
