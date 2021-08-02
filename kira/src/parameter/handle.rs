use std::sync::Arc;

use crate::{
	error::CommandError,
	manager::command::{producer::CommandProducer, Command, ParameterCommand},
};

use super::{ParameterId, ParameterShared, Tween};

/// Controls a parameter.
///
/// When a [`ParameterHandle`] is dropped, the corresponding parameter
/// will be removed.
pub struct ParameterHandle {
	pub(crate) id: ParameterId,
	pub(crate) shared: Arc<ParameterShared>,
	pub(crate) command_producer: CommandProducer,
}

impl ParameterHandle {
	/// Returns the unique identifier for the parameter.
	pub fn id(&self) -> ParameterId {
		self.id
	}

	/// Returns the current value of the parameter.
	pub fn value(&self) -> f64 {
		self.shared.value()
	}

	/// Returns `true` if the parameter is paused and `false`
	/// if not.
	pub fn paused(&self) -> bool {
		self.shared.paused()
	}

	/// Pauses the parameter, preventing tweens from advancing.
	pub fn pause(&mut self) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Parameter(ParameterCommand::Pause(self.id)))
	}

	/// Resumes the parameter, allowing tweens to advance.
	pub fn resume(&mut self) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Parameter(ParameterCommand::Resume(self.id)))
	}

	/// Smoothly transitions the parameter to a new value with the
	/// specified tween.
	pub fn set(&mut self, target: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Parameter(ParameterCommand::Set {
				id: self.id,
				target,
				tween,
			}))
	}
}

impl Drop for ParameterHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}
