//! Provides an interface for controlling multiple instances
//! and sequences at a time.
//!
//! Groups can be created with [`AudioManager::add_group`](crate::manager::AudioManager::add_group).
//! [`Sound`](crate::sound::Sound)s, [`Arrangement`](crate::arrangement::Arrangement)s
//! and [`Sequence`](crate::sequence::Sequence)s can be assigned
//! to any number of groups when they're created.
//! Groups themselves can also be assigned to groups.
//!
//! [`pause_group`](crate::manager::AudioManager::pause_group),
//! [`resume_group`](crate::manager::AudioManager::resume_group), and
//! [`stop_group`](crate::manager::AudioManager::stop_group) will
//! affect all instances that have the specified group anywhere in
//! their ancestry.

pub(crate) mod groups;
mod handle;
mod set;

pub use handle::GroupHandle;
pub use set::GroupSet;
use uuid::Uuid;

use crate::util::generate_uuid;

/**
A unique identifier for a group.

You cannot create this manually - a group ID is created
when you create a group with an [`AudioManager`](crate::manager::AudioManager).
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
pub struct GroupId {
	uuid: Uuid,
}

impl GroupId {
	pub(crate) fn new() -> Self {
		Self {
			uuid: generate_uuid(),
		}
	}
}

impl From<&GroupHandle> for GroupId {
	fn from(handle: &GroupHandle) -> Self {
		handle.id()
	}
}

/// Settings for a group.
#[derive(Debug, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct GroupSettings {
	/// The unique identifier for the group.
	pub id: GroupId,
	/// The groups this group belongs to.
	pub groups: GroupSet,
}

impl GroupSettings {
	/// Creates a new `GroupSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the unique identifier for the group.
	pub fn id(self, id: impl Into<GroupId>) -> Self {
		Self {
			id: id.into(),
			..Default::default()
		}
	}

	/// Sets the groups this group belongs to.
	pub fn groups(self, groups: impl Into<GroupSet>) -> Self {
		Self {
			groups: groups.into(),
			..Default::default()
		}
	}
}

impl Default for GroupSettings {
	fn default() -> Self {
		Self {
			id: GroupId::new(),
			groups: GroupSet::new(),
		}
	}
}

#[derive(Debug, Clone)]
pub(crate) struct Group {
	groups: GroupSet,
}

impl Group {
	pub fn new(settings: GroupSettings) -> Self {
		Self {
			groups: settings.groups,
		}
	}

	pub fn groups(&self) -> &GroupSet {
		&self.groups
	}
}
