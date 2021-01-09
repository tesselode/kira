use indexmap::IndexMap;

use crate::command::GroupCommand;

use super::{Group, GroupId};

pub(crate) struct Groups {
	groups: IndexMap<GroupId, Group>,
}

impl Groups {
	pub fn new(capacity: usize) -> Self {
		Self {
			groups: IndexMap::with_capacity(capacity),
		}
	}

	pub fn get(&self, id: GroupId) -> Option<&Group> {
		self.groups.get(&id)
	}

	pub fn run_command(&mut self, command: GroupCommand) -> Option<Group> {
		match command {
			GroupCommand::AddGroup(id, group) => self.groups.insert(id, group),
			GroupCommand::RemoveGroup(id) => self.groups.remove(&id),
		}
	}
}
