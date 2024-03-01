//! [`Arena`] iterators.

use crate::arena::{slot::ArenaSlotState, Arena, Key};

/// Iterates over shared references to the items in
/// the [`Arena`].
///
/// The most recently added items will be visited first.
pub struct Iter<'a, T> {
	next_occupied_slot_index: Option<usize>,
	arena: &'a Arena<T>,
}

impl<'a, T> Iter<'a, T> {
	pub(super) fn new(arena: &'a Arena<T>) -> Self {
		Self {
			next_occupied_slot_index: arena.first_occupied_slot_index,
			arena,
		}
	}
}

impl<'a, T> Iterator for Iter<'a, T> {
	type Item = (Key, &'a T);

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(index) = self.next_occupied_slot_index {
			let slot = &self.arena.slots[index];
			if let ArenaSlotState::Occupied {
				data,
				next_occupied_slot_index,
				..
			} = &slot.state
			{
				self.next_occupied_slot_index = *next_occupied_slot_index;
				Some((
					Key {
						index,
						generation: slot.generation,
					},
					data,
				))
			} else {
				panic!("the iterator should not encounter a free slot");
			}
		} else {
			None
		}
	}
}

/// Iterates over mutable references to the items in
/// the [`Arena`].
///
/// The most recently added items will be visited first.
pub struct IterMut<'a, T> {
	next_occupied_slot_index: Option<usize>,
	arena: &'a mut Arena<T>,
}

impl<'a, T> IterMut<'a, T> {
	pub(super) fn new(arena: &'a mut Arena<T>) -> Self {
		Self {
			next_occupied_slot_index: arena.first_occupied_slot_index,
			arena,
		}
	}
}

impl<'a, T> Iterator for IterMut<'a, T> {
	type Item = (Key, &'a mut T);

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(index) = self.next_occupied_slot_index {
			let slot = &mut self.arena.slots[index];
			if let ArenaSlotState::Occupied {
				data,
				next_occupied_slot_index,
				..
			} = &mut slot.state
			{
				self.next_occupied_slot_index = *next_occupied_slot_index;
				Some((
					Key {
						index,
						generation: slot.generation,
					},
					// using a small bit of unsafe code here to get around
					// borrow checker limitations. this workaround is stolen
					// from slotmap: https://github.com/orlp/slotmap/blob/master/src/hop.rs#L1165
					unsafe {
						let data: *mut T = &mut *data;
						&mut *data
					},
				))
			} else {
				panic!("the iterator should not encounter a free slot");
			}
		} else {
			None
		}
	}
}

/// An iterator that removes and yields elements from an
/// [`Arena`] according to a filter function.
pub struct DrainFilter<'a, T, F: FnMut(&T) -> bool> {
	arena: &'a mut Arena<T>,
	filter: F,
	next_occupied_slot_index: Option<usize>,
}

impl<'a, T, F: FnMut(&T) -> bool> DrainFilter<'a, T, F> {
	pub(super) fn new(arena: &'a mut Arena<T>, filter: F) -> Self {
		Self {
			next_occupied_slot_index: arena.first_occupied_slot_index,
			arena,
			filter,
		}
	}
}

impl<'a, T, F: FnMut(&T) -> bool> Iterator for DrainFilter<'a, T, F> {
	type Item = (Key, T);

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(index) = self.next_occupied_slot_index {
			let slot = &mut self.arena.slots[index];
			if let ArenaSlotState::Occupied {
				data,
				next_occupied_slot_index,
				..
			} = &mut slot.state
			{
				self.next_occupied_slot_index = *next_occupied_slot_index;
				if (self.filter)(data) {
					let key = Key {
						index,
						generation: slot.generation,
					};
					return self
						.arena
						.remove_from_slot(index)
						.map(|element| (key, element));
				}
			} else {
				panic!("the iterator should not encounter a free slot");
			}
		}
		None
	}
}
