use crate::{group::GroupSet, mixer::TrackIndex};

use super::ArrangementId;

/// Settings for an [`Arrangement`](crate::arrangement::Arrangement).
#[derive(Debug, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct ArrangementSettings {
	/// The unique identifier for this arrangement.
	pub id: ArrangementId,
	/// The track instances of this arrangement will play on by default.
	pub default_track: TrackIndex,
	/// Whether the arrangement should have a "cool off" period after playing
	/// before it can be played again, and if so, the duration
	/// of that cool off period.
	///
	/// This is useful to avoid situations where the same arrangement
	/// is played multiple times at the exact same point in time,
	/// resulting in the arrangement being louder than normal.
	pub cooldown: Option<f64>,
	/// How long the arrangement is musically.
	///
	/// For example, a recording of a 2-bar drum fill
	/// in an echoey cathedral may have 5 seconds of actual
	/// drumming and then 10 seconds of reverberations from
	/// the building. So even though the audio is 15 seconds
	/// long, you might say the music only lasts for 5 seconds.
	///
	/// If set, the semantic duration of the arrangement will be
	/// used as the default end point when looping the arrangement.
	pub semantic_duration: Option<f64>,
	/// Whether the arrangement should be looped by default, and if so,
	/// the point an instance should jump back to when it reaches
	/// the end.
	pub default_loop_start: Option<f64>,
	/// The groups this arrangement belongs to.
	pub groups: GroupSet,
}

impl ArrangementSettings {
	/// Creates a new `ArrangementSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the unique identifier for the arrangement.
	pub fn id(self, id: impl Into<ArrangementId>) -> Self {
		Self {
			id: id.into(),
			..self
		}
	}

	/// Sets the track instances of this arrangement will play on by default.
	pub fn default_track<T: Into<TrackIndex>>(self, track: T) -> Self {
		Self {
			default_track: track.into(),
			..self
		}
	}

	/// Sets the cooldown time of the arrangement.
	pub fn cooldown(self, cooldown: f64) -> Self {
		Self {
			cooldown: Some(cooldown),
			..self
		}
	}

	/// Sets the semantic duration of the arrangement.
	pub fn semantic_duration(self, semantic_duration: f64) -> Self {
		Self {
			semantic_duration: Some(semantic_duration),
			..self
		}
	}

	/// Sets the default loop start point of the arrangement.
	pub fn default_loop_start(self, default_loop_start: f64) -> Self {
		Self {
			default_loop_start: Some(default_loop_start),
			..self
		}
	}

	/// Sets the group this arrangement belongs to.
	pub fn groups(self, groups: impl Into<GroupSet>) -> Self {
		Self {
			groups: groups.into(),
			..self
		}
	}
}

impl Default for ArrangementSettings {
	fn default() -> Self {
		Self {
			id: ArrangementId::new(),
			default_track: TrackIndex::Main,
			cooldown: Some(0.0001),
			semantic_duration: None,
			default_loop_start: None,
			groups: GroupSet::new(),
		}
	}
}

/// Settings for a looping [`Arrangement`](super::Arrangement).
#[derive(Debug, Clone)]
pub struct LoopArrangementSettings {
	/// The unique identifier for this arrangement.
	pub id: ArrangementId,
	/// The track instances of this arrangement will play on by default.
	pub default_track: TrackIndex,
	/// Whether the arrangement should have a "cool off" period after playing
	/// before it can be played again, and if so, the duration
	/// of that cool off period.
	///
	/// This is useful to avoid situations where the same arrangement
	/// is played multiple times at the exact same point in time,
	/// resulting in the arrangement being louder than normal.
	pub cooldown: Option<f64>,
	/// How long the arrangement is musically.
	///
	/// For example, a recording of a 2-bar drum fill
	/// in an echoey cathedral may have 5 seconds of actual
	/// drumming and then 10 seconds of reverberations from
	/// the building. So even though the audio is 15 seconds
	/// long, you might say the music only lasts for 5 seconds.
	///
	/// If set, the semantic duration of the arrangement will be
	/// used as the default end point when looping the arrangement.
	pub semantic_duration: Option<f64>,
	/// The groups this arrangement belongs to.
	pub groups: GroupSet,
}

impl LoopArrangementSettings {
	/// Creates a new `LoopArrangementSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the unique identifier for the arrangement.
	pub fn id(self, id: impl Into<ArrangementId>) -> Self {
		Self {
			id: id.into(),
			..self
		}
	}

	/// Sets the track instances of this arrangement will play on by default.
	pub fn default_track<T: Into<TrackIndex>>(self, track: T) -> Self {
		Self {
			default_track: track.into(),
			..self
		}
	}

	/// Sets the cooldown time of the arrangement.
	pub fn cooldown(self, cooldown: f64) -> Self {
		Self {
			cooldown: Some(cooldown),
			..self
		}
	}

	/// Sets the semantic duration of the arrangement.
	pub fn semantic_duration(self, semantic_duration: f64) -> Self {
		Self {
			semantic_duration: Some(semantic_duration),
			..self
		}
	}

	/// Sets the group this arrangement belongs to.
	pub fn groups(self, groups: impl Into<GroupSet>) -> Self {
		Self {
			groups: groups.into(),
			..self
		}
	}
}

impl Default for LoopArrangementSettings {
	fn default() -> Self {
		Self {
			id: ArrangementId::new(),
			default_track: TrackIndex::Main,
			cooldown: Some(0.0001),
			semantic_duration: None,
			groups: GroupSet::new(),
		}
	}
}
