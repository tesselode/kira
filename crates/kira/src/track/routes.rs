use std::collections::HashMap;

use crate::value::{CachedValue, Value};

use super::TrackId;

/// Defines how the output of a mixer sub-track will be
/// fed into the input of other mixer tracks.
pub struct TrackRoutes(HashMap<TrackId, Value>);

impl TrackRoutes {
	/// Creates a new [`TrackRoutes`] with the default settings.
	///
	/// By default, a mixer track will send its output to the
	/// main mixer track at full volume.
	pub fn new() -> Self {
		Self({
			let mut routes = HashMap::new();
			routes.insert(TrackId::Main, Value::Fixed(1.0));
			routes
		})
	}

	/// Sets how much of the current track's signal will be sent
	/// to the specified destination track.
	pub fn with_route(mut self, track: impl Into<TrackId>, volume: impl Into<Value>) -> Self {
		self.0.insert(track.into(), volume.into());
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
