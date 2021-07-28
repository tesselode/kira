use std::ops::{RangeFrom, RangeFull, RangeInclusive, RangeToInclusive};

use crate::manager::resources::parameters::Parameters;

use super::Value;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValidRange {
	pub lower_bound: Option<f64>,
	pub upper_bound: Option<f64>,
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

pub struct CachedValue {
	valid_range: ValidRange,
	value: Value,
	raw_value: f64,
}

impl CachedValue {
	pub fn new(valid_range: impl Into<ValidRange>, value: Value, default: f64) -> Self {
		Self {
			valid_range: valid_range.into(),
			value,
			raw_value: default,
		}
	}

	pub(crate) fn get(&self) -> f64 {
		self.raw_value
	}

	pub(crate) fn set(&mut self, value: Value) {
		self.value = value;
		if let Value::Fixed(raw_value) = self.value {
			self.raw_value = raw_value;
		}
	}

	pub(crate) fn update(&mut self, parameters: &Parameters) {
		if let Value::Parameter { id, mapping } = self.value {
			if let Some(parameter) = parameters.get(id) {
				self.raw_value = mapping.map(parameter.value());
			}
		}
	}
}
