pub mod groups;

use std::sync::atomic::{AtomicUsize, Ordering};

use groups::Groups;
use indexmap::IndexSet;

use crate::{
	arrangement::ArrangementId, instance::InstanceId, playable::Playable,
	sequence::SequenceInstanceId, sound::SoundId,
};

static NEXT_GROUP_INDEX: AtomicUsize = AtomicUsize::new(0);

/**
A unique identifier for a group.

You cannot create this manually - a group ID is created
when you create a group with an [`AudioManager`](crate::manager::AudioManager).
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct GroupId {
	index: usize,
}

impl GroupId {
	pub(crate) fn new() -> Self {
		let index = NEXT_GROUP_INDEX.fetch_add(1, Ordering::Relaxed);
		Self { index }
	}
}

#[derive(Debug, Clone)]
pub(crate) struct Group {
	groups: IndexSet<GroupId>,
}

impl Group {
	pub fn new(groups: IndexSet<GroupId>) -> Self {
		Self { groups }
	}

	/// Returns if this group is in the group with the given ID.
	pub fn is_in_group(&self, parent_id: GroupId, groups: &Groups) -> bool {
		// check if this group is a direct descendant of the requested group
		if self.groups.contains(&parent_id) {
			return true;
		}
		// otherwise, recursively check if any of the direct parents of this
		// group is in the requested group
		for id in &self.groups {
			if let Some(group) = groups.get(*id) {
				if group.is_in_group(parent_id, groups) {
					return true;
				}
			}
		}
		false
	}

	pub fn add_to_group(&mut self, id: GroupId) {
		self.groups.insert(id);
	}

	pub fn remove_from_group(&mut self, id: GroupId) {
		self.groups.remove(&id);
	}
}

pub enum Groupable {
	Group(GroupId),
	Playable(Playable),
}

impl From<GroupId> for Groupable {
	fn from(id: GroupId) -> Self {
		Self::Group(id)
	}
}

impl From<Playable> for Groupable {
	fn from(playable: Playable) -> Self {
		Self::Playable(playable)
	}
}

impl From<SoundId> for Groupable {
	fn from(id: SoundId) -> Self {
		Self::Playable(Playable::Sound(id))
	}
}

impl From<ArrangementId> for Groupable {
	fn from(id: ArrangementId) -> Self {
		Self::Playable(Playable::Arrangement(id))
	}
}
