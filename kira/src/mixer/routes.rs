use std::collections::HashMap;

use crate::value::{cached::CachedValue, Value};

use super::TrackId;

pub struct TrackRoutes(HashMap<TrackId, Value>);

impl TrackRoutes {
	pub fn new() -> Self {
		Self({
			let mut routes = HashMap::new();
			routes.insert(TrackId::Main, Value::Fixed(1.0));
			routes
		})
	}

	pub fn with_route(mut self, track: impl Into<TrackId>, value: impl Into<Value>) -> Self {
		self.0.insert(track.into(), value.into());
		self
	}

	pub(crate) fn into_vec(self) -> Vec<(TrackId, CachedValue)> {
		self.0
			.iter()
			.map(|(id, value)| (*id, CachedValue::new(.., *value, 0.0)))
			.collect()
	}
}

impl Default for TrackRoutes {
	fn default() -> Self {
		Self::new()
	}
}
