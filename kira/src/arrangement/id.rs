use std::hash::Hash;

use uuid::Uuid;

use crate::util::generate_uuid;

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
			uuid: generate_uuid(),
		}
	}
}

impl From<&ArrangementHandle> for ArrangementId {
	fn from(handle: &ArrangementHandle) -> Self {
		handle.id()
	}
}
