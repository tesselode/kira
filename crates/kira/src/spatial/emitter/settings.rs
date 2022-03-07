use crate::tween::Easing;

use super::EmitterDistances;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmitterSettings {
	pub distances: EmitterDistances,
	pub attenuation_function: Option<Easing>,
}

impl EmitterSettings {
	pub fn new() -> Self {
		Self {
			distances: EmitterDistances::default(),
			attenuation_function: Some(Easing::Linear),
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
}

impl Default for EmitterSettings {
	fn default() -> Self {
		Self::new()
	}
}
