use std::hash::Hash;

use uuid::Uuid;

use crate::util::generate_uuid;

use super::SoundHandle;

/// A unique identifier for a [`Sound`](crate::sound::Sound).
///
/// You cannot create this manually - a sound ID is returned
/// when you add a sound to an [`AudioManager`](crate::manager::AudioManager).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SoundId {
	uuid: Uuid,
}

impl SoundId {
	pub(crate) fn new() -> Self {
		Self {
			uuid: generate_uuid(),
		}
	}
}

impl From<&SoundHandle> for SoundId {
	fn from(handle: &SoundHandle) -> Self {
		handle.id()
	}
}
