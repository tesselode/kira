pub(crate) mod groups;

use std::sync::atomic::{AtomicUsize, Ordering};

use groups::Groups;
use indexmap::IndexSet;

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
}
