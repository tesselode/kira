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

	pub fn get_mut(&mut self, id: GroupId) -> Option<&mut Group> {
		self.groups.get_mut(&id)
	}

	pub fn run_command(&mut self, command: GroupCommand) -> Option<Group> {
		match command {
			GroupCommand::AddGroup(id, group) => {
				self.groups.insert(id, group);
			}
			GroupCommand::RemoveGroup(id) => {
				return self.groups.remove(&id);
			}
			GroupCommand::AddToGroup(id_a, id_b) => {
				if let Some(group) = self.groups.get_mut(&id_a) {
					group.add_to_group(id_b);
				}
			}
			GroupCommand::RemoveFromGroup(id_a, id_b) => {
				if let Some(group) = self.groups.get_mut(&id_a) {
					group.remove_from_group(id_b);
				}
			}
		}
		None
	}
}
