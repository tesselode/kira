use crate::{manager::backend::parameters::Parameters, parameter::ParameterId};

/// A number that something can be set to.
///
/// Can either be a fixed number or the current value
/// of a parameter.
#[derive(Debug, Copy, Clone)]
pub enum Value {
	Fixed(f64),
	Parameter(ParameterId),
}

impl Value {
	pub(crate) fn get(&self, parameters: &Parameters) -> Option<f64> {
		match self {
			Value::Fixed(value) => Some(*value),
			Value::Parameter(id) => parameters.get(*id).map(|parameter| parameter.value()),
		}
	}
}

impl From<f64> for Value {
	fn from(value: f64) -> Self {
		Self::Fixed(value)
	}
}

impl From<ParameterId> for Value {
	fn from(id: ParameterId) -> Self {
		Self::Parameter(id)
	}
}

#[derive(Debug, Clone)]
pub(crate) struct CachedValue {
	value: Value,
	last_value: f64,
}

impl CachedValue {
	pub fn new(value: Value, default_value: f64) -> Self {
		Self {
			value,
			last_value: default_value,
		}
	}

	pub fn set(&mut self, value: Value) {
		self.value = value;
	}

	pub fn update(&mut self, parameters: &Parameters) {
		if let Some(value) = self.value.get(parameters) {
			self.last_value = value;
		}
	}

	pub fn value(&self) -> f64 {
		self.last_value
	}
}
