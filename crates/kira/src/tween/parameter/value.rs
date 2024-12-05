use std::{
	ops::{Add, Div, Mul, Neg, Rem, Sub},
	time::Duration,
};

use crate::{
	info::Info,
	modulator::ModulatorId,
	tween::{Easing, Tweenable},
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
		mapping: Mapping<T>,
	},
}

impl<T> Value<T> {
	/// Creates a `Value::FromModulator` from a modulator ID or handle.
	#[must_use]
	pub fn from_modulator(id: impl Into<ModulatorId>, mapping: Mapping<T>) -> Self {
		Self::FromModulator {
			id: id.into(),
			mapping,
		}
	}

	/// Converts a `Value<T>` to a `Value<T2>`.
	#[must_use = "This method returns a new Value and does not mutate the original."]
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
	pub(crate) fn raw_value(self, info: &Info) -> Option<T> {
		match self {
			Value::Fixed(value) => Some(value),
			Value::FromModulator { id, mapping } => {
				info.modulator_value(id).map(|value| mapping.map(value))
			}
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

impl<T> From<T> for Value<mint::Vector3<f32>>
where
	T: Into<mint::Vector3<f32>>,
{
	fn from(value: T) -> Self {
		Self::Fixed(value.into())
	}
}

impl<T> From<T> for Value<mint::Quaternion<f32>>
where
	T: Into<mint::Quaternion<f32>>,
{
	fn from(value: T) -> Self {
		Self::Fixed(value.into())
	}
}

impl From<Duration> for Value<Duration> {
	fn from(value: Duration) -> Self {
		Self::Fixed(value)
	}
}

impl<T: Default> Default for Value<T> {
	fn default() -> Self {
		Self::Fixed(T::default())
	}
}

impl<T, Rhs> Add<Rhs> for Value<T>
where
	T: Add<Rhs, Output = T>,
	Rhs: Copy,
{
	type Output = Self;

	fn add(self, rhs: Rhs) -> Self::Output {
		match self {
			Value::Fixed(value) => Self::Fixed(value + rhs),
			Value::FromModulator { id, mapping } => Value::FromModulator {
				id,
				mapping: mapping.add_output(rhs),
			},
		}
	}
}

impl<T, Rhs> Sub<Rhs> for Value<T>
where
	T: Sub<Rhs, Output = T>,
	Rhs: Copy,
{
	type Output = Self;

	fn sub(self, rhs: Rhs) -> Self::Output {
		match self {
			Value::Fixed(value) => Self::Fixed(value - rhs),
			Value::FromModulator { id, mapping } => Value::FromModulator {
				id,
				mapping: mapping.sub_output(rhs),
			},
		}
	}
}

impl<T, Rhs> Mul<Rhs> for Value<T>
where
	T: Mul<Rhs, Output = T>,
	Rhs: Copy,
{
	type Output = Self;

	fn mul(self, rhs: Rhs) -> Self::Output {
		match self {
			Value::Fixed(value) => Self::Fixed(value * rhs),
			Value::FromModulator { id, mapping } => Value::FromModulator {
				id,
				mapping: mapping.mul_output(rhs),
			},
		}
	}
}

impl<T, Rhs> Div<Rhs> for Value<T>
where
	T: Div<Rhs, Output = T>,
	Rhs: Copy,
{
	type Output = Self;

	fn div(self, rhs: Rhs) -> Self::Output {
		match self {
			Value::Fixed(value) => Self::Fixed(value / rhs),
			Value::FromModulator { id, mapping } => Value::FromModulator {
				id,
				mapping: mapping.div_output(rhs),
			},
		}
	}
}

impl<T, Rhs> Rem<Rhs> for Value<T>
where
	T: Rem<Rhs, Output = T>,
	Rhs: Copy,
{
	type Output = Self;

	fn rem(self, rhs: Rhs) -> Self::Output {
		match self {
			Value::Fixed(value) => Self::Fixed(value % rhs),
			Value::FromModulator { id, mapping } => Value::FromModulator {
				id,
				mapping: mapping.rem_output(rhs),
			},
		}
	}
}

impl<T> Neg for Value<T>
where
	T: Neg<Output = T>,
{
	type Output = Self;

	fn neg(self) -> Self::Output {
		match self {
			Value::Fixed(value) => Self::Fixed(-value),
			Value::FromModulator { id, mapping } => Value::FromModulator {
				id,
				mapping: mapping.neg_output(),
			},
		}
	}
}

/// A transformation from a modulator's value to a parameter value.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mapping<T> {
	/// A range of values from a modulator.
	pub input_range: (f64, f64),
	/// The corresponding range of values of the parameter.
	pub output_range: (T, T),
	/// The curve to apply to the output.
	pub easing: Easing,
}

impl<T> Mapping<T> {
	/// Converts a `Mapping<T>` to a `Mapping<T2>`.
	#[must_use = "This method returns a new Mapping and does not mutate the original."]
	pub fn to_<T2: From<T>>(self) -> Mapping<T2> {
		Mapping {
			input_range: self.input_range,
			output_range: (self.output_range.0.into(), self.output_range.1.into()),
			easing: self.easing,
		}
	}

	/// Transforms an input value to an output value using this mapping.
	#[must_use]
	pub fn map(self, input: f64) -> T
	where
		T: Tweenable,
	{
		let mut amount = (input - self.input_range.0) / (self.input_range.1 - self.input_range.0);
		amount = amount.clamp(0.0, 1.0);
		amount = self.easing.apply(amount);
		T::interpolate(self.output_range.0, self.output_range.1, amount)
	}

	/// Adds `rhs` to the minimum and maximum values of the output range.
	pub fn add_output<Rhs>(self, rhs: Rhs) -> Self
	where
		T: Add<Rhs, Output = T>,
		Rhs: Copy,
	{
		Self {
			output_range: (self.output_range.0 + rhs, self.output_range.1 + rhs),
			..self
		}
	}

	/// Subtracts `rhs` from the minimum and maximum values of the output range.
	pub fn sub_output<Rhs>(self, rhs: Rhs) -> Self
	where
		T: Sub<Rhs, Output = T>,
		Rhs: Copy,
	{
		Self {
			output_range: (self.output_range.0 - rhs, self.output_range.1 - rhs),
			..self
		}
	}

	/// Multiplies the minimum and maximum values of the output range by `rhs`.
	pub fn mul_output<Rhs>(self, rhs: Rhs) -> Self
	where
		T: Mul<Rhs, Output = T>,
		Rhs: Copy,
	{
		Self {
			output_range: (self.output_range.0 * rhs, self.output_range.1 * rhs),
			..self
		}
	}

	/// Divides the minimum and maximum values of the output range by `rhs`.
	pub fn div_output<Rhs>(self, rhs: Rhs) -> Self
	where
		T: Div<Rhs, Output = T>,
		Rhs: Copy,
	{
		Self {
			output_range: (self.output_range.0 / rhs, self.output_range.1 / rhs),
			..self
		}
	}

	/// Sets the minimum and maximum values of the output range to `x % rhs`.
	pub fn rem_output<Rhs>(self, rhs: Rhs) -> Self
	where
		T: Rem<Rhs, Output = T>,
		Rhs: Copy,
	{
		Self {
			output_range: (self.output_range.0 % rhs, self.output_range.1 % rhs),
			..self
		}
	}

	/// Negates the minimum and maximum values of the output range.
	pub fn neg_output(self) -> Self
	where
		T: Neg<Output = T>,
	{
		Self {
			output_range: (-self.output_range.0, -self.output_range.1),
			..self
		}
	}
}
