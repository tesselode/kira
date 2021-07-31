use crate::value::Value;

use super::{effect::Effect, routes::TrackRoutes};

pub struct TrackSettings {
	pub volume: Value,
	pub panning: Value,
	pub routes: TrackRoutes,
	pub effects: Vec<Box<dyn Effect>>,
}

impl TrackSettings {
	pub fn new() -> Self {
		Self {
			volume: Value::Fixed(1.0),
			panning: Value::Fixed(0.5),
			routes: TrackRoutes::new(),
			effects: vec![],
		}
	}

	pub fn volume(self, volume: impl Into<Value>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	pub fn panning(self, panning: impl Into<Value>) -> Self {
		Self {
			panning: panning.into(),
			..self
		}
	}

	pub fn routes(self, routes: TrackRoutes) -> Self {
		Self { routes, ..self }
	}

	pub fn with_effect(mut self, effect: impl Effect + 'static) -> Self {
		self.effects.push(Box::new(effect));
		self
	}
}

impl Default for TrackSettings {
	fn default() -> Self {
		Self::new()
	}
}
