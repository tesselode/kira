//! [`Arena`] iterators.

use crate::arena::{
	slot::{ArenaSlot, ArenaSlotState},
	Arena, Key,
};
use core::marker::PhantomData;

/// Iterates over shared references to the items in
/// the [`Arena`].
///
/// The most recently added items will be visited first.
pub struct Iter<'a, T> {
	next_occupied_slot_index: Option<u16>,
	arena: &'a Arena<T>,
}

impl<'a, T> Iter<'a, T> {
	#[must_use]
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
			let slot = &self.arena.slots[index as usize];
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
	next_occupied_slot_index: Option<u16>,
	slots: *mut [ArenaSlot<T>],
	marker: PhantomData<&'a mut Arena<T>>,
}

impl<'a, T> IterMut<'a, T> {
	#[must_use]
	pub(super) fn new(arena: &'a mut Arena<T>) -> Self {
		Self {
			next_occupied_slot_index: arena.first_occupied_slot_index,
			slots: arena.slots.as_mut_slice(),
			marker: PhantomData,
		}
	}
}

impl<'a, T> Iterator for IterMut<'a, T> {
	type Item = (Key, &'a mut T);

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(index) = self.next_occupied_slot_index {
			let index_usize = usize::from(index);
			let slot = {
				// as_mut_ptr and get_unchecked_mut on *mut [T] are unstable :(
				let start_ptr = self.slots.cast::<ArenaSlot<T>>();
				// SAFETY: This is always in bounds.
				let slot_ptr = unsafe { start_ptr.add(index_usize) };
				// SAFETY:
				// * This relies on the invariant that `next_occupied_slot_index` never repeats. If
				//   it did repeat, we could create aliasing mutable references here.
				// * Lifetime is the same that we mutably borrow the Arena for.
				unsafe { slot_ptr.as_mut::<'a>() }.unwrap()
			};

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

/// An iterator that removes and yields elements from an
/// [`Arena`] according to a filter function.
pub struct DrainFilter<'a, T, F: FnMut(&T) -> bool> {
	arena: &'a mut Arena<T>,
	filter: F,
	next_occupied_slot_index: Option<u16>,
}

impl<'a, T, F: FnMut(&T) -> bool> DrainFilter<'a, T, F> {
	#[must_use]
	pub(super) fn new(arena: &'a mut Arena<T>, filter: F) -> Self {
		Self {
			next_occupied_slot_index: arena.first_occupied_slot_index,
			arena,
			filter,
		}
	}
}

impl<T, F: FnMut(&T) -> bool> Iterator for DrainFilter<'_, T, F> {
	type Item = (Key, T);

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(index) = self.next_occupied_slot_index {
			let slot = &mut self.arena.slots[index as usize];
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
