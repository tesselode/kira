use crate::{
	command::ParameterCommand,
	parameter::{Parameter, ParameterId},
	static_container::index_map::StaticIndexMap,
};

/// A collection of all of the currently active parameters.
///
/// This is mainly used internally - you only need to use this
/// if you're writing your own effects.
#[derive(Debug, Clone)]
pub struct Parameters {
	parameters: StaticIndexMap<ParameterId, Parameter>,
}

impl Parameters {
	pub(crate) fn new(capacity: usize) -> Self {
		Self {
			parameters: StaticIndexMap::new(capacity),
		}
	}

	pub(crate) fn get(&self, id: ParameterId) -> Option<&Parameter> {
		self.parameters.get(&id)
	}

	pub(crate) fn run_command(&mut self, command: ParameterCommand) {
		match command {
			ParameterCommand::AddParameter(id, value) => {
				self.parameters.try_insert(id, Parameter::new(value)).ok();
			}
			ParameterCommand::SetParameter(id, value, tween) => {
				if let Some(parameter) = self.parameters.get_mut(&id) {
					parameter.set(value, tween);
				}
			}
			ParameterCommand::RemoveParameter(id) => {
				self.parameters.remove(&id);
			}
		}
	}

	pub(crate) fn update(&mut self, dt: f64) {
		for (_, parameter) in &mut self.parameters {
			parameter.update(dt);
		}
	}
}
