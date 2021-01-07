use std::hash::Hash;

use uuid::Uuid;

use crate::util::generate_uuid;

use super::ArrangementHandle;

/**
A unique identifier for an [`Arrangement`](Arrangement).

You cannot create this manually - an arrangement ID is created
when you [add an arrangement](crate::manager::AudioManager::add_arrangement)
to an [`AudioManager`](crate::manager::AudioManager).
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
