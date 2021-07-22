use std::hash::Hash;

use uuid::Uuid;

use super::handle::SoundHandle;

/// A unique identifier for a [`Sound`](crate::sound::Sound).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
pub struct SoundId {
	uuid: Uuid,
}

impl SoundId {
	pub(crate) fn new() -> Self {
		Self {
			uuid: Uuid::new_v4(),
		}
	}
}

impl From<&SoundHandle> for SoundId {
	fn from(handle: &SoundHandle) -> Self {
		handle.id()
	}
}
