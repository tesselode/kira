use std::hash::Hash;

use uuid::Uuid;

use super::ArrangementHandle;

/// A unique identifier for an [`Arrangement`](super::Arrangement).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
pub struct ArrangementId {
	uuid: Uuid,
}

impl ArrangementId {
	pub(crate) fn new() -> Self {
		Self {
			uuid: Uuid::new_v4(),
		}
	}
}

impl From<&ArrangementHandle> for ArrangementId {
	fn from(handle: &ArrangementHandle) -> Self {
		handle.id()
	}
}
