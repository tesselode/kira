#![warn(missing_docs)]

mod controller;
pub mod error;
pub mod iter;
mod slot;

#[cfg(test)]
mod test;

pub use controller::Controller;

use error::{ArenaFull, InsertWithKeyError};
use iter::{DrainFilter, Iter, IterMut};
use slot::{ArenaSlot, ArenaSlotState};

/// A unique identifier for an item in an [`Arena`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
	index: u16,
	generation: u32,
}

/// A container of items that can be accessed via a [`Key`].
#[derive(Debug)]
pub struct Arena<T> {
	controller: Controller,
	slots: Vec<ArenaSlot<T>>,
	first_occupied_slot_index: Option<u16>,
}

impl<T> Arena<T> {
	/// Creates a new [`Arena`] with enough space for `capacity`
	/// number of items.
	#[must_use]
	pub fn new(capacity: u16) -> Self {
		Self {
			controller: Controller::new(capacity),
			slots: (0..capacity).map(|_| ArenaSlot::new()).collect(),
			first_occupied_slot_index: None,
		}
	}

	/// Returns a [`Controller`] for this [`Arena`].
	#[must_use]
	pub fn controller(&self) -> Controller {
		self.controller.clone()
	}

	/// Returns the total capacity for this [`Arena`].
	#[must_use]
	pub fn capacity(&self) -> usize {
		self.slots.len()
	}

	/// Returns the number of items currently in the [`Arena`].
	#[must_use]
	pub fn len(&self) -> usize {
		self.slots
			.iter()
			.filter(|slot| matches!(&slot.state, ArenaSlotState::Occupied { .. }))
			.count()
	}

	/// Returns `true` if the [`Arena`] is currently empty.
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Tries to insert an item into the [`Arena`] with a previously
	/// reserved [`Key`].
	pub fn insert_with_key(&mut self, key: Key, data: T) -> Result<(), InsertWithKeyError> {
		// make sure the key is valid and reserved
		if let Some(slot) = self.slots.get(key.index as usize) {
			if slot.generation != key.generation {
				return Err(InsertWithKeyError::InvalidKey);
			}
			if let ArenaSlotState::Occupied { .. } = &slot.state {
				return Err(InsertWithKeyError::KeyNotReserved);
			}
		} else {
			return Err(InsertWithKeyError::InvalidKey);
		}

		// update the previous head to point to the new head
		// as the previous occupied slot
		if let Some(head_index) = self.first_occupied_slot_index {
			self.slots[head_index as usize].set_previous_occupied_slot_index(Some(key.index));
		}

		// insert the new data
		self.slots[key.index as usize].state = ArenaSlotState::Occupied {
			data,
			previous_occupied_slot_index: None,
			next_occupied_slot_index: self.first_occupied_slot_index,
		};

		// update the head
		self.first_occupied_slot_index = Some(key.index);

		Ok(())
	}

	/// Tries to reserve a [`Key`], and, if successful, inserts
	/// an item into the [`Arena`] with that [`Key`] and
	/// returns the [`Key`].
	pub fn insert(&mut self, data: T) -> Result<Key, ArenaFull> {
		let key = self.controller.try_reserve()?;
		self.insert_with_key(key, data).unwrap();
		Ok(key)
	}

	fn remove_from_slot(&mut self, index: u16) -> Option<T> {
		let slot = &mut self.slots[index as usize];
		let state = std::mem::replace(&mut slot.state, ArenaSlotState::Free);
		match state {
			ArenaSlotState::Free => None,
			ArenaSlotState::Occupied {
				data,
				previous_occupied_slot_index,
				next_occupied_slot_index,
			} => {
				slot.generation += 1;
				self.controller.free(index);

				// update the pointers of the previous and next slots
				if let Some(previous_index) = previous_occupied_slot_index {
					self.slots[previous_index as usize]
						.set_next_occupied_slot_index(next_occupied_slot_index);
				}
				if let Some(next_index) = next_occupied_slot_index {
					self.slots[next_index as usize]
						.set_previous_occupied_slot_index(previous_occupied_slot_index);
				}

				// update the head if needed.
				//
				// `first_occupied_slot_index` should always be `Some` in this case,
				// because this branch of the `match` is only taken if the slot is
				// occupied, and if any slots are occupied, `first_occupied_slot_index`
				// should be `Some`. if not, there's a major bug that needs addressing.
				if self.first_occupied_slot_index.unwrap() == index {
					self.first_occupied_slot_index = next_occupied_slot_index;
				}

				Some(data)
			}
		}
	}

	/// If the [`Arena`] contains an item with the given [`Key`],
	/// removes it from the [`Arena`] and returns `Some(item)`.
	/// Otherwise, returns `None`.
	#[must_use]
	pub fn remove(&mut self, key: Key) -> Option<T> {
		// TODO: answer the following questions:
		// - if you reserve a key, then try to remove the key
		// without having inserted anything, should the slot
		// be unreserved? the current answer is no
		// - what should happen if you try to remove a slot
		// with the wrong generation? currently the answer is
		// it just returns None like normal
		let slot = &mut self.slots[key.index as usize];
		if slot.generation != key.generation {
			return None;
		}
		self.remove_from_slot(key.index)
	}

	/// Returns a shared reference to the item in the [`Arena`] with
	/// the given [`Key`] if it exists. Otherwise, returns `None`.
	#[must_use]
	pub fn get(&self, key: Key) -> Option<&T> {
		let slot = &self.slots[key.index as usize];
		if slot.generation != key.generation {
			return None;
		}
		match &slot.state {
			ArenaSlotState::Free => None,
			ArenaSlotState::Occupied { data, .. } => Some(data),
		}
	}

	/// Returns a mutable reference to the item in the [`Arena`] with
	/// the given [`Key`] if it exists. Otherwise, returns `None`.
	#[must_use]
	pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
		let slot = &mut self.slots[key.index as usize];
		if slot.generation != key.generation {
			return None;
		}
		match &mut slot.state {
			ArenaSlotState::Free => None,
			ArenaSlotState::Occupied { data, .. } => Some(data),
		}
	}

	/// Retains only the elements specified by the predicate.
	///
	/// In other words, remove all elements e such that f(&e) returns false.
	pub fn retain(&mut self, mut f: impl FnMut(&T) -> bool) {
		let mut index = match self.first_occupied_slot_index {
			Some(index) => index,
			None => return,
		};
		loop {
			if let ArenaSlotState::Occupied {
				data,
				next_occupied_slot_index,
				..
			} = &self.slots[index as usize].state
			{
				let next_occupied_slot_index = next_occupied_slot_index.as_ref().copied();
				if !f(data) {
					self.remove_from_slot(index);
				}
				index = match next_occupied_slot_index {
					Some(index) => index,
					None => return,
				}
			} else {
				panic!("expected the slot pointed to by first_occupied_slot_index/next_occupied_slot_index to be occupied")
			}
		}
	}

	/// Returns an iterator over shared references to the items in
	/// the [`Arena`].
	///
	/// The most recently added items will be visited first.
	#[must_use]
	pub fn iter(&self) -> Iter<T> {
		Iter::new(self)
	}

	/// Returns an iterator over mutable references to the items in
	/// the [`Arena`].
	///
	/// The most recently added items will be visited first.
	#[must_use]
	pub fn iter_mut(&mut self) -> IterMut<T> {
		IterMut::new(self)
	}

	/// Returns an iterator that removes and yields all elements
	/// for which `filter(&element)` returns `true`.
	#[must_use]
	pub fn drain_filter<F: FnMut(&T) -> bool>(&mut self, filter: F) -> DrainFilter<T, F> {
		DrainFilter::new(self, filter)
	}
}

impl<T> std::ops::Index<Key> for Arena<T> {
	type Output = T;

	fn index(&self, key: Key) -> &Self::Output {
		self.get(key).expect("No item associated with this key")
	}
}

impl<T> std::ops::IndexMut<Key> for Arena<T> {
	fn index_mut(&mut self, key: Key) -> &mut Self::Output {
		self.get_mut(key).expect("No item associated with this key")
	}
}

impl<'a, T> IntoIterator for &'a Arena<T> {
	type Item = (Key, &'a T);

	type IntoIter = Iter<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T> IntoIterator for &'a mut Arena<T> {
	type Item = (Key, &'a mut T);

	type IntoIter = IterMut<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}
