//! Saves the last valid raw value of a [`Value`]. Useful for writing
//! [`Effect`](crate::track::effect::Effect)s and
//! [`AudioStream`](crate::audio_stream::AudioStream)s.

use std::ops::{RangeFrom, RangeFull, RangeInclusive, RangeToInclusive};

use crate::manager::resources::Parameters;

use super::Value;

/// The valid range of raw values for a [`CachedValue`].
///
/// Both bounds are always inclusive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValidRange {
	/// The lower bound of the range.
	pub lower_bound: Option<f64>,
	/// The upper bound of the range.
	pub upper_bound: Option<f64>,
}

impl ValidRange {
	pub(crate) fn clamp(&mut self, mut x: f64) -> f64 {
		if let Some(lower_bound) = self.lower_bound {
			x = x.max(lower_bound);
		}
		if let Some(upper_bound) = self.upper_bound {
			x = x.min(upper_bound);
		}
		x
	}
}

impl From<RangeInclusive<f64>> for ValidRange {
	fn from(range: RangeInclusive<f64>) -> Self {
		Self {
			lower_bound: Some(*range.start()),
			upper_bound: Some(*range.end()),
		}
	}
}

impl From<RangeFrom<f64>> for ValidRange {
	fn from(range: RangeFrom<f64>) -> Self {
		Self {
			lower_bound: Some(range.start),
			upper_bound: None,
		}
	}
}

impl From<RangeToInclusive<f64>> for ValidRange {
	fn from(range: RangeToInclusive<f64>) -> Self {
		Self {
			lower_bound: None,
			upper_bound: Some(range.end),
		}
	}
}

impl From<RangeFull> for ValidRange {
	fn from(_: RangeFull) -> Self {
		Self {
			lower_bound: None,
			upper_bound: None,
		}
	}
}

/// Holds a [`Value`] and remembers the last valid raw value.
pub struct CachedValue {
	valid_range: ValidRange,
	value: Value,
	raw_value: f64,
}

impl CachedValue {
	/// Creates a new [`CachedValue`].
	pub fn new(valid_range: impl Into<ValidRange>, value: Value, default: f64) -> Self {
		Self {
			valid_range: valid_range.into(),
			value,
			raw_value: match value {
				Value::Fixed(value) => value,
				Value::Parameter { .. } => default,
			},
		}
	}

	/// Gets the last valid raw value.
	pub fn get(&self) -> f64 {
		self.raw_value
	}

	/// Sets the value.
	pub fn set(&mut self, value: Value) {
		self.value = value;
		if let Value::Fixed(raw_value) = self.value {
			self.raw_value = self.valid_range.clamp(raw_value);
		}
	}

	/// Updates the [`CachedValue`] with the current values of parameters.
	pub fn update(&mut self, parameters: &Parameters) {
		if let Value::Parameter { id, mapping } = self.value {
			if let Some(parameter) = parameters.get(id) {
				self.raw_value = self.valid_range.clamp(mapping.map(parameter.value()));
			}
		}
	}
}
