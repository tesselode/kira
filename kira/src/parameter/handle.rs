//! An interface for controlling parameters.

use crate::command::{
	producer::{CommandError, CommandProducer},
	ParameterCommand,
};

use super::{tween::Tween, ParameterId};

#[derive(Debug, Clone)]
/// Allows you to control a parameter.
pub struct ParameterHandle {
	id: ParameterId,
	command_sender: CommandProducer,
}

impl ParameterHandle {
	pub(crate) fn new(id: ParameterId, command_sender: CommandProducer) -> Self {
		Self { id, command_sender }
	}

	/// Returns the ID of the parameter.
	pub fn id(&self) -> ParameterId {
		self.id
	}

	/// Sets the parameter to a value with an optional tween.
	pub fn set(&mut self, value: f64, tween: impl Into<Option<Tween>>) -> Result<(), CommandError> {
		self.command_sender
			.push(ParameterCommand::SetParameter(self.id, value, tween.into()).into())
	}
}
