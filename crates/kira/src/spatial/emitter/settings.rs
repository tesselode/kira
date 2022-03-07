use crate::tween::Easing;

use super::EmitterDistances;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmitterSettings {
	pub distances: EmitterDistances,
	pub attenuation_function: Option<Easing>,
	pub enable_spatialization: bool,
}

impl EmitterSettings {
	pub fn new() -> Self {
		Self {
			distances: EmitterDistances::default(),
			attenuation_function: Some(Easing::Linear),
			enable_spatialization: true,
		}
	}

	pub fn distances(self, distances: impl Into<EmitterDistances>) -> Self {
		Self {
			distances: distances.into(),
			..self
		}
	}

	pub fn attenuation_function(self, attenuation_function: impl Into<Option<Easing>>) -> Self {
		Self {
			attenuation_function: attenuation_function.into(),
			..self
		}
	}

	pub fn enable_spatialization(self, enable_spatialization: bool) -> Self {
		Self {
			enable_spatialization,
			..self
		}
	}
}

impl Default for EmitterSettings {
	fn default() -> Self {
		Self::new()
	}
}
