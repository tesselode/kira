use std::collections::HashMap;

use crate::{tween::Tweener, Volume};

use super::TrackId;

/// Defines how the output of a mixer sub-track will be
/// fed into the input of other mixer tracks.
pub struct TrackRoutes(pub(crate) HashMap<TrackId, Volume>);

impl TrackRoutes {
	/// Creates a new [`TrackRoutes`] with the default settings.
	///
	/// By default, a mixer track will send its output to the
	/// main mixer track at full volume.
	pub fn new() -> Self {
		Self::parent(TrackId::Main)
	}

	/// Creates a new [`TrackRoutes`] with no routes pre-configured.
	///
	/// If you set a track's routes to this as is, you will not hear
	/// any audio output from that track, since it is not routed
	/// to the main track nor to any other track.
	pub fn empty() -> Self {
		Self(HashMap::new())
	}

	/// Creates a new [`TrackRoutes`] with a single route to the
	/// specified track.
	pub fn parent(track: impl Into<TrackId>) -> Self {
		Self({
			let mut routes = HashMap::new();
			routes.insert(track.into(), Volume::Amplitude(1.0));
			routes
		})
	}

	/// Sets how much of the current track's signal will be sent
	/// to the specified destination track.
	pub fn with_route(mut self, track: impl Into<TrackId>, volume: impl Into<Volume>) -> Self {
		self.0.insert(track.into(), volume.into());
		self
	}

	/// Removes the route to the specified track.
	pub fn without_route(mut self, track: impl Into<TrackId>) -> Self {
		self.0.remove(&track.into());
		self
	}

	pub(crate) fn into_vec(self) -> Vec<(TrackId, Tweener<Volume>)> {
		self.0
			.iter()
			.map(|(id, value)| (*id, Tweener::new(*value)))
			.collect()
	}
}

impl Default for TrackRoutes {
	fn default() -> Self {
		Self::new()
	}
}
