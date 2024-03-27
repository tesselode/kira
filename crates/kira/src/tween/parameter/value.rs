use std::time::Duration;

use crate::tween::Tweenable;

/// A value that a parameter can be linked to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value<T> {
	/// A fixed value.
	Fixed(T),
}

impl<T> Value<T> {
	/// Converts a `Value<T>` to a `Value<T2>`.
	pub fn to_<T2: From<T>>(self) -> Value<T2> {
		match self {
			Value::Fixed(value) => Value::Fixed(value.into()),
		}
	}
}

impl<T> Value<T>
where
	T: Tweenable,
{
	pub(crate) fn raw_value(self) -> Option<T> {
		match self {
			Value::Fixed(value) => Some(value),
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
