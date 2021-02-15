use basedrop::Owned;
use indexmap::IndexMap;

use crate::command::GroupCommand;

use super::{Group, GroupId};

pub(crate) struct Groups {
	groups: IndexMap<GroupId, Owned<Group>>,
}

impl Groups {
	pub fn new(capacity: usize) -> Self {
		Self {
			groups: IndexMap::with_capacity(capacity),
		}
	}

	pub fn get(&self, id: GroupId) -> Option<&Owned<Group>> {
		self.groups.get(&id)
	}

	pub fn run_command(&mut self, command: GroupCommand) {
		match command {
			GroupCommand::AddGroup(id, group) => {
				self.groups.insert(id, group);
			}
			GroupCommand::RemoveGroup(id) => {
				self.groups.remove(&id);
			}
		}
	}
}
