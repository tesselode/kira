//! Provides a wrapper around sounds and arrangements.

use indexmap::IndexMap;

use crate::{
	arrangement::{Arrangement, ArrangementHandle, ArrangementId},
	group::{groups::Groups, GroupId},
	mixer::TrackIndex,
	sound::{Sound, SoundHandle, SoundId},
	Frame,
};

/// Represents something you can play multiple instances of.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Playable {
	/// A sound.
	Sound(SoundId),
	/// An arrangement.
	Arrangement(ArrangementId),
}

impl Playable {
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
