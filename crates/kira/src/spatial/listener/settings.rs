use crate::track::TrackId;

pub struct ListenerSettings {
	pub track: TrackId,
}

impl ListenerSettings {
	pub fn new() -> Self {
		Self {
			track: TrackId::Main,
		}
	}

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
