//! Provides a wrapper around sounds and arrangements.

use std::vec;

use indexmap::IndexMap;

use crate::{
	arrangement::{Arrangement, ArrangementHandle, ArrangementId},
	group::{groups::Groups, GroupId},
	mixer::TrackIndex,
	sound::{Sound, SoundHandle, SoundId},
	Frame,
};

/// Settings for a [`Playable`](Playable) item.
#[derive(Debug, Clone)]
pub struct PlayableSettings {
	/// The track instances of this item will play on by default.
	pub default_track: TrackIndex,
	/// Whether the item should have a "cool off" period after playing
	/// before it can be played again, and if so, the duration
	/// of that cool off period.
	///
	/// This is useful to avoid situations where the same item
	/// is played multiple times at the exact same point in time,
	/// resulting in the item being louder than normal.
	pub cooldown: Option<f64>,
	/// How long the item is musically.
	///
	/// For example, a recording of a 2-bar drum fill
	/// in an echoey cathedral may have 5 seconds of actual
	/// drumming and then 10 seconds of reverberations from
	/// the building. So even though the audio is 15 seconds
	/// long, you might say the music only lasts for 5 seconds.
	///
	/// If set, the semantic duration of the item will be
	/// used as the default end point when looping the item.
	pub semantic_duration: Option<f64>,
	/// Whether the item should be looped by default, and if so,
	/// the point an instance should jump back to when it reaches
	/// the end.
	pub default_loop_start: Option<f64>,
	/// The groups this item belongs to.
	pub groups: Vec<GroupId>,
}

impl PlayableSettings {
	/// Creates a new `PlayableSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the track instances of this item will play on by default.
	pub fn default_track<T: Into<TrackIndex>>(self, track: T) -> Self {
		Self {
			default_track: track.into(),
			..self
		}
	}

	/// Sets the cooldown time of the item.
	pub fn cooldown(self, cooldown: f64) -> Self {
		Self {
			cooldown: Some(cooldown),
			..self
		}
	}

	/// Sets the semantic duration of the item.
	pub fn semantic_duration(self, semantic_duration: f64) -> Self {
		Self {
			semantic_duration: Some(semantic_duration),
			..self
		}
	}

	/// Sets the default loop start point of the item.
	pub fn default_loop_start(self, default_loop_start: f64) -> Self {
		Self {
			default_loop_start: Some(default_loop_start),
			..self
		}
	}

	/// Sets the group this item belongs to.
	pub fn groups<T: Into<Vec<GroupId>>>(self, groups: T) -> Self {
		Self {
			groups: groups.into(),
			..self
		}
	}
}

impl Default for PlayableSettings {
	fn default() -> Self {
		Self {
			default_track: TrackIndex::Main,
			cooldown: Some(0.0001),
			semantic_duration: None,
			default_loop_start: None,
			groups: vec![],
		}
	}
}

/// Represents something you can play multiple instances of.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Playable {
	/// A sound.
	Sound(SoundId),
	/// An arrangement.
	Arrangement(ArrangementId),
}

impl Playable {
	/// Gets the duration of the item.
	pub fn duration(&self) -> f64 {
		match self {
			Playable::Sound(id) => id.duration(),
			Playable::Arrangement(id) => id.duration(),
		}
	}

	/// Gets the default track instances of this item will play on.
	pub fn default_track(&self) -> TrackIndex {
		match self {
			Playable::Sound(id) => id.default_track(),
			Playable::Arrangement(id) => id.default_track(),
		}
	}

	/// Gets the [semantic duration](crate::playable::PlayableSettings#structfield.semantic_duration)
	/// of the item.
	pub fn semantic_duration(&self) -> Option<f64> {
		match self {
			Playable::Sound(id) => id.semantic_duration(),
			Playable::Arrangement(id) => id.semantic_duration(),
		}
	}

	/// Gets the default loop start point for instances of this
	/// item.
	pub fn default_loop_start(&self) -> Option<f64> {
		match self {
			Playable::Sound(id) => id.default_loop_start(),
			Playable::Arrangement(id) => id.default_loop_start(),
		}
	}

	/// Gets the frame this item will output at a certain time.
	pub(crate) fn get_frame_at_position(
		&self,
		position: f64,
		sounds: &IndexMap<SoundId, Sound>,
		arrangements: &IndexMap<ArrangementId, Arrangement>,
	) -> Frame {
		match self {
			Playable::Sound(id) => {
				if let Some(sound) = sounds.get(id) {
					sound.get_frame_at_position(position)
				} else {
					Frame::from_mono(0.0)
				}
			}
			Playable::Arrangement(id) => {
				if let Some(arrangement) = arrangements.get(id) {
					arrangement.get_frame_at_position(position, sounds)
				} else {
					Frame::from_mono(0.0)
				}
			}
		}
	}

	pub(crate) fn is_in_group(
		&self,
		parent_id: GroupId,
		sounds: &IndexMap<SoundId, Sound>,
		arrangements: &IndexMap<ArrangementId, Arrangement>,
		groups: &Groups,
	) -> bool {
		match self {
			Playable::Sound(id) => {
				if let Some(sound) = sounds.get(id) {
					return sound.is_in_group(parent_id, groups);
				}
			}
			Playable::Arrangement(id) => {
				if let Some(arrangement) = arrangements.get(id) {
					return arrangement.is_in_group(parent_id, groups);
				}
			}
		}
		false
	}
}

impl From<SoundId> for Playable {
	fn from(id: SoundId) -> Self {
		Self::Sound(id)
	}
}

impl From<ArrangementId> for Playable {
	fn from(id: ArrangementId) -> Self {
		Self::Arrangement(id)
	}
}

impl From<SoundHandle> for Playable {
	fn from(handle: SoundHandle) -> Self {
		Self::Sound(handle.id())
	}
}

impl From<ArrangementHandle> for Playable {
	fn from(handle: ArrangementHandle) -> Self {
		Self::Arrangement(handle.id())
	}
}
