use crate::value::Value;

use super::routes::TrackRoutes;

#[derive(Clone)]
pub struct SubTrackSettings {
	pub(crate) volume: Value<f64>,
	pub(crate) routes: TrackRoutes,
	pub(crate) num_effects: usize,
}

impl SubTrackSettings {
	pub fn new() -> Self {
		Self {
			volume: Value::Fixed(1.0),
			routes: TrackRoutes::default(),
			num_effects: 10,
		}
	}

	pub fn volume(self, volume: impl Into<Value<f64>>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	pub fn routes(self, routes: TrackRoutes) -> Self {
		Self { routes, ..self }
	}

	pub fn num_effects(self, num_effects: usize) -> Self {
		Self {
			num_effects,
			..self
		}
	}
}

impl Default for SubTrackSettings {
	fn default() -> Self {
		Self::new()
	}
}
