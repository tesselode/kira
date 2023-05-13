use std::time::Duration;

use crate::{
	modulator::{value_provider::ModulatorValueProvider, ModulatorId},
	tween::Tweenable,
};

/// A value that a parameter can be linked to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value<T> {
	/// A fixed value.
	Fixed(T),
	/// The value of a [`modulator`](crate::modulator).
	FromModulator {
		/// The modulator to link to.
		id: ModulatorId,
		/// How the modulator's value should be converted to the parameter's value.
		mapping: ModulatorMapping<T>,
	},
}

impl<T> Value<T> {
	/// Creates a `Value::FromModulator` from a modulator ID or handle.
	pub fn from_modulator(id: impl Into<ModulatorId>, mapping: ModulatorMapping<T>) -> Self {
		Self::FromModulator {
			id: id.into(),
			mapping,
		}
	}

	/// Converts a `Value<T>` to a `Value<T2>`.
	pub fn to_<T2: From<T>>(self) -> Value<T2> {
		match self {
			Value::Fixed(value) => Value::Fixed(value.into()),
			Value::FromModulator { id, mapping } => Value::FromModulator {
				id,
				mapping: mapping.to_(),
			},
		}
	}
}

impl<T> Value<T>
where
	T: Tweenable,
{
	pub(crate) fn raw_value(self, modulator_value_provider: &ModulatorValueProvider) -> Option<T> {
		match self {
			Value::Fixed(value) => Some(value),
			Value::FromModulator { id, mapping } => modulator_value_provider
				.get(id)
				.map(|value| mapping.map(value)),
		}
	}
}

impl From<f32> for Value<f32> {
	fn from(value: f32) -> Self {
		Self::Fixed(value)
	}
}

impl From<f64> for Value<f64> {
	fn from(value: f64) -> Self {
		Self::Fixed(value)
	}
}

impl From<mint::Vector3<f32>> for Value<mint::Vector3<f32>> {
	fn from(value: mint::Vector3<f32>) -> Self {
		Self::Fixed(value)
	}
}

impl From<mint::Quaternion<f32>> for Value<mint::Quaternion<f32>> {
	fn from(value: mint::Quaternion<f32>) -> Self {
		Self::Fixed(value)
	}
}

impl From<Duration> for Value<Duration> {
	fn from(value: Duration) -> Self {
		Self::Fixed(value)
	}
}

impl<T, IntoId> From<IntoId> for Value<T>
where
	ModulatorId: From<IntoId>,
	ModulatorMapping<T>: Default,
{
	fn from(id: IntoId) -> Self {
		Self::FromModulator {
			id: id.into(),
			mapping: ModulatorMapping::default(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModulatorMapping<T> {
	pub input_range: (f64, f64),
	pub output_range: (T, T),
	pub clamp_bottom: bool,
	pub clamp_top: bool,
}

impl<T> ModulatorMapping<T> {
	pub fn to_<T2: From<T>>(self) -> ModulatorMapping<T2> {
		ModulatorMapping {
			input_range: self.input_range,
			output_range: (self.output_range.0.into(), self.output_range.1.into()),
			clamp_bottom: self.clamp_bottom,
			clamp_top: self.clamp_top,
		}
	}

	pub fn map(self, input: f64) -> T
	where
		T: Tweenable,
	{
		let mut amount = (input - self.input_range.0) / (self.input_range.1 - self.input_range.0);
		if self.clamp_bottom {
			amount = amount.max(0.0);
		}
		if self.clamp_top {
			amount = amount.min(1.0);
		}
		T::interpolate(self.output_range.0, self.output_range.1, amount)
	}
}

impl Default for ModulatorMapping<f32> {
	fn default() -> Self {
		Self {
			input_range: (0.0, 1.0),
			output_range: (0.0, 1.0),
			clamp_bottom: false,
			clamp_top: false,
		}
	}
}

impl Default for ModulatorMapping<f64> {
	fn default() -> Self {
		Self {
			input_range: (0.0, 1.0),
			output_range: (0.0, 1.0),
			clamp_bottom: false,
			clamp_top: false,
		}
	}
}
