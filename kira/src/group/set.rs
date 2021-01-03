use indexmap::IndexSet;

use super::{groups::Groups, GroupId};

#[derive(Debug, Clone)]
pub struct GroupSet(IndexSet<GroupId>);

impl GroupSet {
	pub fn new() -> Self {
		Self(IndexSet::new())
	}

	pub fn add(mut self, id: impl Into<GroupId>) -> Self {
		self.0.insert(id.into());
		self
	}

	pub fn remove(mut self, id: impl Into<GroupId>) -> Self {
		self.0.remove(&id.into());
		self
	}

	pub fn contains(&self, id: impl Into<GroupId>) -> bool {
		self.0.contains(&id.into())
	}

	/// Returns true if one of the groups in the set has a specified
	/// group as an ancestor or is that group itself.
	pub(crate) fn has_ancestor(&self, ancestor: GroupId, all_groups: &Groups) -> bool {
		// make sure the group actually exists
		if all_groups.get(ancestor).is_none() {
			return false;
		}
		// check if any groups in this set are the target group
		for id in &self.0 {
			if *id == ancestor {
				return true;
			}
		}
		// otherwise, recursively check if the target group
		// is an ancestor of any groups in the set
		for id in &self.0 {
			if let Some(group) = all_groups.get(*id) {
				if group.groups().has_ancestor(ancestor, all_groups) {
					return true;
				}
			}
		}
		false
	}
}
