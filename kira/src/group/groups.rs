use flume::Sender;
use indexmap::IndexMap;

use crate::{command::GroupCommand, resource::Resource};

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

	pub fn run_command(&mut self, command: GroupCommand, unloader: &mut Sender<Resource>) {
		match command {
			GroupCommand::AddGroup(id, group) => {
				if let Some(group) = self.groups.insert(id, group) {
					unloader.try_send(Resource::Group(group)).ok();
				}
			}
			GroupCommand::RemoveGroup(id) => {
				if let Some(group) = self.groups.remove(&id) {
					unloader.try_send(Resource::Group(group)).ok();
				}
			}
		}
	}
}
