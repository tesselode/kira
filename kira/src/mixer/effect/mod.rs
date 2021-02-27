//! Modifies audio in real time.

// TODO: make delay and reverb adapt to the current sample rate

pub mod delay;
pub mod distortion;
pub mod filter;
pub mod handle;
pub mod reverb;

use handle::EffectHandle;

use std::fmt::Debug;

use uuid::Uuid;

use crate::{frame::Frame, parameter::Parameters, Value};

/// A unique identifier for an effect.
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
			uuid: Uuid::new_v4(),
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
	/// The unique identifier for the effect.
	pub id: EffectId,
	/// Whether the effect is initially enabled.
	pub enabled: bool,
	/// The balance between dry (unaffected) signal and wet
	/// (affected) signal to output. 0.0 is fully dry,
	/// 1.0 is fully wet.
	pub mix: Value<f64>,
}

impl EffectSettings {
	/// Creates a new `EffectSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the unique identifier for the effect.
	pub fn id(self, id: impl Into<EffectId>) -> Self {
		Self {
			id: id.into(),
			..self
		}
	}

	/// Sets whether the effect is initially enabled.
	pub fn enabled(self, enabled: bool) -> Self {
		Self { enabled, ..self }
	}

	/// Sets the balance between dry (unaffected) signal and wet
	/// (affected) signal to output. 0.0 is fully dry,
	/// 1.0 is fully wet.
	pub fn mix(self, mix: impl Into<Value<f64>>) -> Self {
		Self {
			mix: mix.into(),
			..self
		}
	}
}

impl Default for EffectSettings {
	fn default() -> Self {
		Self {
			id: EffectId::new(),
			enabled: true,
			mix: Value::Fixed(1.0),
		}
	}
}

/// Receives input audio from a mixer track and outputs modified audio.
pub trait Effect: Send + Debug {
	/// Modifies an input frame.
	/// - `dt` is the time that's elapsed since the previous frame (in seconds)
	/// - `input` is the input audio
	/// - `parameters` is a set of all parameter IDs and their corresponding values.
	/// This is useful in conjunction with [`CachedValue`](crate::CachedValue)s,
	/// which can respond to parameter changes and update their value accordingly.
	fn process(&mut self, dt: f64, input: Frame, parameters: &Parameters) -> Frame;
}
