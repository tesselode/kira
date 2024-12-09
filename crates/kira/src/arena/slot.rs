#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ArenaSlotState<T> {
	Free,
	Occupied {
		data: T,
		previous_occupied_slot_index: Option<u16>,
		next_occupied_slot_index: Option<u16>,
	},
}

#[derive(Debug)]
pub(super) struct ArenaSlot<T> {
	pub(super) state: ArenaSlotState<T>,
	pub(super) generation: u32,
}

impl<T> ArenaSlot<T> {
	#[must_use]
	pub(super) fn new() -> Self {
		Self {
			state: ArenaSlotState::Free,
			generation: 0,
		}
	}

	pub(super) fn set_previous_occupied_slot_index(&mut self, index: Option<u16>) {
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

	pub(super) fn set_next_occupied_slot_index(&mut self, index: Option<u16>) {
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
