//! The [`Value`] enum, which is used as the type for many settings
//! throughout Kira.

mod cached;
mod mapping;

pub use cached::*;
pub use mapping::*;

use crate::parameter::{ParameterHandle, ParameterId};

/// The possible values for a setting.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Value {
	/// The setting is fixed to the specified value.
	Fixed(f64),
	/// The setting is linked to a parameter with the
	/// given mapping.
	Parameter {
		/// The parameter the setting is linked to.
		id: ParameterId,
		/// The mapping of parameter values to setting values.
		mapping: Mapping,
	},
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
