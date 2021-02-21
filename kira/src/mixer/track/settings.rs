use super::{sends::TrackSends, SendTrackId, SubTrackId, TrackIndex};

/// Settings for a mixer sub-track.
#[derive(Debug, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct SubTrackSettings {
	/// The unique identifier for the track.
	pub id: SubTrackId,
	/// The track that this track's output will be routed to.
	pub parent_track: TrackIndex,
	/// The send tracks that this track will be routed to (in
	/// addition to the parent track).
	pub sends: TrackSends,
	/// The volume of the track.
	pub volume: f64,
	/// The maximum number of effects this track can hold.
	pub num_effects: usize,
}

impl SubTrackSettings {
	/// Creates a new `TrackSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the unique identifier for the track.
	pub fn id(self, id: impl Into<SubTrackId>) -> Self {
		Self {
			id: id.into(),
			..self
		}
	}

	/// Sets the track that this track's output will be routed to.
	pub fn parent_track(self, parent_track: impl Into<TrackIndex>) -> Self {
		Self {
			parent_track: parent_track.into(),
			..self
		}
	}

	/// Sets the send tracks that this track will be routed to (in
	/// addition to the parent track).
	pub fn sends(self, sends: TrackSends) -> Self {
		Self { sends, ..self }
	}

	/// Sets the volume of the track.
	pub fn volume(self, volume: impl Into<f64>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/// Sets the maximum number of effects this track can hold.
	pub fn num_effects(self, num_effects: usize) -> Self {
		Self {
			num_effects,
			..self
		}
	}
}

impl Default for SubTrackSettings {
	fn default() -> Self {
		Self {
			id: SubTrackId::new(),
			parent_track: TrackIndex::Main,
			sends: TrackSends::new(),
			volume: 1.0,
			num_effects: 10,
		}
	}
}

/// Settings for a mixer send-track.
#[derive(Debug, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct SendTrackSettings {
	/// The unique identifier for the track.
	pub id: SendTrackId,
	/// The volume of the track.
	pub volume: f64,
	/// The maximum number of effects this track can hold.
	pub num_effects: usize,
}

impl SendTrackSettings {
	/// Creates a new `TrackSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the unique identifier for the track.
	pub fn id(self, id: impl Into<SendTrackId>) -> Self {
		Self {
			id: id.into(),
			..self
		}
	}

	/// Sets the volume of the track.
	pub fn volume(self, volume: impl Into<f64>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/// Sets the maximum number of effects this track can hold.
	pub fn num_effects(self, num_effects: usize) -> Self {
		Self {
			num_effects,
			..self
		}
	}
}

impl Default for SendTrackSettings {
	fn default() -> Self {
		Self {
			id: SendTrackId::new(),
			volume: 1.0,
			num_effects: 10,
		}
	}
}
