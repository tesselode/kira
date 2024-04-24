use crate::track::TrackId;

/// Settings for a listener.
pub struct ListenerSettings {
	/// The mixer track that the listener's received audio should be routed to.
	pub track: TrackId,
}

impl ListenerSettings {
	/// Creates a new [`ListenerSettings`] with the default settings.
	pub fn new() -> Self {
		Self {
			track: TrackId::Main,
		}
	}

	/// Sets the mixer track that the listener's received audio should be routed to.
	pub fn track(self, track: impl Into<TrackId>) -> Self {
		Self {
			track: track.into(),
			..self
		}
	}
}

impl Default for ListenerSettings {
	fn default() -> Self {
		Self::new()
	}
}
