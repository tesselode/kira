use crate::parameter::{Mapping, ParameterId, Parameters};

/// A number that something can be set to.
///
/// Can either be a fixed number or the current value
/// of a parameter.
#[derive(Debug, Copy, Clone)]
pub enum Value<T: From<f64> + Copy> {
	Fixed(T),
	Parameter(ParameterId, Mapping),
}

impl<T: From<f64> + Copy> Value<T> {
	pub(crate) fn get(&self, parameters: &Parameters) -> Option<T> {
		match self {
			Value::Fixed(value) => Some(*value),
			Value::Parameter(id, mapping) => parameters
				.get(*id)
				.map(|parameter| T::from(mapping.map(parameter.value()))),
		}
	}
}

impl<T: From<f64> + Copy> From<T> for Value<T> {
	fn from(value: T) -> Self {
		Self::Fixed(value)
	}
}

impl<T: From<f64> + Copy> From<ParameterId> for Value<T> {
	fn from(id: ParameterId) -> Self {
		Self::Parameter(id, Mapping::default())
	}
}

/// A wrapper around [`Value`](crate::Value)s that remembers the last valid raw value.
///
/// You'll only need to use this if you're writing your own effects.
#[derive(Debug, Copy, Clone)]
pub struct CachedValue<T: From<f64> + Copy> {
	value: Value<T>,
	last_value: T,
}

impl<T: From<f64> + Copy> CachedValue<T> {
	/// Creates a `CachedValue` with an initial value setting
	/// and a default raw value to fall back on.
	pub fn new(value: Value<T>, default_value: T) -> Self {
		Self {
			value,
			last_value: default_value,
		}
	}

	/// Sets the value.
	pub fn set(&mut self, value: Value<T>) {
		self.value = value;
	}

	/// If the value is set to a parameter, updates the raw value
	/// from the parameter (if it exists).
	pub fn update(&mut self, parameters: &Parameters) {
		if let Some(value) = self.value.get(parameters) {
			self.last_value = value;
		}
	}

	/// Gets the last valid raw value.
	pub fn value(&self) -> T {
		self.last_value
	}
}
