//! Provides a wrapper around sounds and arrangements.

mod playables;

pub(crate) use playables::Playables;

use crate::{
	arrangement::{handle::ArrangementHandle, Arrangement, ArrangementId},
	group::{groups::Groups, GroupId},
	mixer::TrackIndex,
	sound::{handle::SoundHandle, Sound, SoundId},
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum Playable<'a> {
	Sound(&'a Sound),
	Arrangement(&'a Arrangement),
}

impl<'a> Playable<'a> {
	pub fn duration(&self) -> f64 {
		match self {
			Playable::Sound(sound) => sound.duration(),
			Playable::Arrangement(arrangement) => arrangement.duration(),
		}
	}

	pub fn default_track(&self) -> TrackIndex {
		match self {
			Playable::Sound(sound) => sound.default_track(),
			Playable::Arrangement(arrangement) => arrangement.default_track(),
		}
	}

	pub fn default_loop_start(&self) -> Option<f64> {
		match self {
			Playable::Sound(sound) => sound.default_loop_start(),
			Playable::Arrangement(arrangement) => arrangement.default_loop_start(),
		}
	}

	pub fn is_in_group(&self, id: GroupId, all_groups: &Groups) -> bool {
		match self {
			Playable::Sound(sound) => sound.is_in_group(id, all_groups),
			Playable::Arrangement(arrangement) => arrangement.is_in_group(id, all_groups),
		}
	}
}

#[derive(Debug)]
pub(crate) enum PlayableMut<'a> {
	Sound(&'a mut Sound),
	Arrangement(&'a mut Arrangement),
}

impl<'a> PlayableMut<'a> {
	pub fn cooling_down(&self) -> bool {
		match self {
			PlayableMut::Sound(sound) => sound.cooling_down(),
			PlayableMut::Arrangement(arrangement) => arrangement.cooling_down(),
		}
	}

	pub fn start_cooldown(&mut self) {
		match self {
			PlayableMut::Sound(sound) => {
				sound.start_cooldown();
			}
			PlayableMut::Arrangement(arrangement) => {
				arrangement.start_cooldown();
			}
		}
	}
}
