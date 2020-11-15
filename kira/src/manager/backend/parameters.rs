use indexmap::IndexMap;

use crate::{
	command::ParameterCommand,
	parameter::{Parameter, ParameterId},
};

#[derive(Debug, Clone)]
pub struct Parameters {
	parameters: IndexMap<ParameterId, Parameter>,
}

impl Parameters {
	pub(crate) fn new(capacity: usize) -> Self {
		Self {
			parameters: IndexMap::with_capacity(capacity),
		}
	}

	pub(crate) fn get(&self, id: ParameterId) -> Option<&Parameter> {
		self.parameters.get(&id)
	}

	pub(crate) fn run_command(&mut self, command: ParameterCommand) {
		match command {
			ParameterCommand::AddParameter(id, value) => {
				let parameter = Parameter::new(value);
				self.parameters.insert(id, parameter);
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
