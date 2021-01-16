pub mod filter;
pub mod handle;

use handle::EffectHandle;

use std::fmt::Debug;

use uuid::Uuid;

use crate::{frame::Frame, parameter::Parameters, util::generate_uuid};

/**
A unique identifier for an effect.

You cannot create this manually - an effect ID is created
when you add an effect to a mixer track with an [`AudioManager`](crate::manager::AudioManager).
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
pub struct EffectId {
	uuid: Uuid,
}

impl EffectId {
	pub(crate) fn new() -> Self {
		Self {
			uuid: generate_uuid(),
		}
	}
}

impl From<&EffectHandle> for EffectId {
	fn from(handle: &EffectHandle) -> Self {
		handle.id()
	}
}

/// Settings for an effect.
#[derive(Debug, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct EffectSettings {
	pub id: EffectId,
	/// Whether the effect is initially enabled.
	pub enabled: bool,
}

impl EffectSettings {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn id(self, id: impl Into<EffectId>) -> Self {
		Self {
			id: id.into(),
			..self
		}
	}

	pub fn enabled(self, enabled: bool) -> Self {
		Self { enabled, ..self }
	}
}

impl Default for EffectSettings {
	fn default() -> Self {
		Self {
			id: EffectId::new(),
			enabled: true,
		}
	}
}

pub trait Effect: Send + Debug {
	fn process(&mut self, dt: f64, input: Frame, parameters: &Parameters) -> Frame;
}
