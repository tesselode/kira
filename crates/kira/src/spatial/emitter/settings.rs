use crate::tween::Easing;

use super::EmitterDistances;

/// Settings for an emitter.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct EmitterSettings {
	/// The distances from a listener at which the emitter is loudest and quietest.
	pub distances: EmitterDistances,
	/// How the emitter's volume will change with distance.
	///
	/// If `None`, the emitter will output at a constant volume.
	pub attenuation_function: Option<Easing>,
	/// Whether the emitter's output should be panned left or right depending on its
	/// direction from the listener.
	pub enable_spatialization: bool,
	/// Whether the emitter should be kept alive until all sounds are finished
	/// playing on it even if the handle is dropped.
	pub persist_until_sounds_finish: bool,
}

impl EmitterSettings {
	/// Creates a new [`EmitterSettings`] with the default settings.
	pub fn new() -> Self {
		Self {
			distances: EmitterDistances::default(),
			attenuation_function: Some(Easing::Linear),
			enable_spatialization: true,
			persist_until_sounds_finish: false,
		}
	}

	/// Sets the distances from a listener at which the emitter is loudest and quietest.
	pub fn distances(self, distances: impl Into<EmitterDistances>) -> Self {
		Self {
			distances: distances.into(),
			..self
		}
	}

	/// Sets how the emitter's volume will change with distance.
	///
	/// If `None`, the emitter will output at a constant volume.
	pub fn attenuation_function(self, attenuation_function: impl Into<Option<Easing>>) -> Self {
		Self {
			attenuation_function: attenuation_function.into(),
			..self
		}
	}

	/// Sets whether the emitter's output should be panned left or right depending on its
	/// direction from the listener.
	pub fn enable_spatialization(self, enable_spatialization: bool) -> Self {
		Self {
			enable_spatialization,
			..self
		}
	}

	/// Sets whether the emitter should be kept alive until all sounds are finished
	/// playing on it even if the handle is dropped.
	pub fn persist_until_sounds_finish(self, persist_until_sounds_finish: bool) -> Self {
		Self {
			persist_until_sounds_finish,
			..self
		}
	}
}

impl Default for EmitterSettings {
	fn default() -> Self {
		Self::new()
	}
}
