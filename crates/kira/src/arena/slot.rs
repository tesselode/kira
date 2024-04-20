#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArenaSlotState<T> {
	Free,
	Occupied {
		data: T,
		previous_occupied_slot_index: Option<u16>,
		next_occupied_slot_index: Option<u16>,
	},
}

#[derive(Debug)]
pub(crate) struct ArenaSlot<T> {
	pub(crate) state: ArenaSlotState<T>,
	pub(crate) generation: u32,
}

impl<T> ArenaSlot<T> {
	pub(crate) fn new() -> Self {
		Self {
			state: ArenaSlotState::Free,
			generation: 0,
		}
	}

	pub(crate) fn set_previous_occupied_slot_index(&mut self, index: Option<u16>) {
		if let ArenaSlotState::Occupied {
			previous_occupied_slot_index,
			..
		} = &mut self.state
		{
			*previous_occupied_slot_index = index;
		} else {
			panic!("expected a slot to be occupied, but it was not");
		}
	}

	pub(crate) fn set_next_occupied_slot_index(&mut self, index: Option<u16>) {
		if let ArenaSlotState::Occupied {
			next_occupied_slot_index,
			..
		} = &mut self.state
		{
			*next_occupied_slot_index = index;
		} else {
			panic!("expected a slot to be occupied, but it was not");
		}
	}
}
