//! Provides a wrapper around sounds and arrangements.

use indexmap::IndexMap;

use crate::{
	arrangement::{Arrangement, ArrangementHandle, ArrangementId},
	group::{groups::Groups, GroupId},
	sound::{Sound, SoundHandle, SoundId},
	Frame,
};

/// Represents something you can play multiple instances of.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize)
)]
pub enum PlayableId {
	/// A sound.
	Sound(SoundId),
	/// An arrangement.
	Arrangement(ArrangementId),
}

impl PlayableId {
	/// Gets the frame this item will output at a certain time.
	pub(crate) fn get_frame_at_position(
		&self,
		position: f64,
		sounds: &IndexMap<SoundId, Sound>,
		arrangements: &IndexMap<ArrangementId, Arrangement>,
	) -> Frame {
		match self {
			PlayableId::Sound(id) => {
				if let Some(sound) = sounds.get(id) {
					sound.get_frame_at_position(position)
				} else {
					Frame::from_mono(0.0)
				}
			}
			PlayableId::Arrangement(id) => {
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
			PlayableId::Sound(id) => {
				if let Some(sound) = sounds.get(id) {
					return sound.is_in_group(parent_id, groups);
				}
			}
			PlayableId::Arrangement(id) => {
				if let Some(arrangement) = arrangements.get(id) {
					return arrangement.is_in_group(parent_id, groups);
				}
			}
		}
		false
	}
}

impl From<SoundId> for PlayableId {
	fn from(id: SoundId) -> Self {
		Self::Sound(id)
	}
}

impl From<ArrangementId> for PlayableId {
	fn from(id: ArrangementId) -> Self {
		Self::Arrangement(id)
	}
}

impl From<&SoundHandle> for PlayableId {
	fn from(handle: &SoundHandle) -> Self {
		Self::Sound(handle.id())
	}
}

impl From<&ArrangementHandle> for PlayableId {
	fn from(handle: &ArrangementHandle) -> Self {
		Self::Arrangement(handle.id())
	}
}
