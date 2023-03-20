use crate::{
	modulator::{value_provider::ModulatorValueProvider, ModulatorId},
	tween::Tweenable,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value<T> {
	Fixed(T),
	FromModulator {
		id: ModulatorId,
		mapping: ModulatorMapping<T>,
	},
}

impl<T> Value<T> {
	pub fn from_modulator(id: impl Into<ModulatorId>, mapping: ModulatorMapping<T>) -> Self {
		Self::FromModulator {
			id: id.into(),
			mapping,
		}
	}

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

impl<T> From<T> for Value<T> {
	fn from(value: T) -> Self {
		Self::Fixed(value)
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
