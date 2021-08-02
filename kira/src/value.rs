pub mod cached;
mod mapping;

use crate::parameter::{ParameterHandle, ParameterId};

pub use mapping::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
	Fixed(f64),
	Parameter { id: ParameterId, mapping: Mapping },
}

impl From<f64> for Value {
	fn from(value: f64) -> Self {
		Self::Fixed(value)
	}
}

impl From<ParameterId> for Value {
	fn from(id: ParameterId) -> Self {
		Self::Parameter {
			id,
			mapping: Default::default(),
		}
	}
}

impl From<&ParameterHandle> for Value {
	fn from(handle: &ParameterHandle) -> Self {
		Self::Parameter {
			id: handle.id(),
			mapping: Default::default(),
		}
	}
}
