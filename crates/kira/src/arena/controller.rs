use std::sync::{
	atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering},
	Arc,
};

use crate::arena::{ArenaFull, Key};

/// Represents that a [`ControllerSlot`] does not have a free slot
/// after it.
///
/// This is used because the next free slot variable is an
/// [`AtomicU16`], but we still need some way to represent the
/// absence of a next free slot.
const NO_NEXT_FREE_SLOT: u16 = u16::MAX;

#[derive(Debug)]
struct ControllerSlot {
	free: AtomicBool,
	generation: AtomicU32,
	next_free_slot_index: AtomicU16,
}

/// The shared state for all [`Controller`]s for an [`Arena`](super::Arena).
#[derive(Debug)]
struct ControllerInner {
	slots: Vec<ControllerSlot>,
	first_free_slot_index: AtomicU16,
}

impl ControllerInner {
	fn new(capacity: u16) -> Self {
		Self {
			slots: (0..capacity)
				.map(|i| ControllerSlot {
					free: AtomicBool::new(true),
					generation: AtomicU32::new(0),
					next_free_slot_index: AtomicU16::new(if i < capacity - 1 {
						i + 1
					} else {
						NO_NEXT_FREE_SLOT
					}),
				})
				.collect(),
			first_free_slot_index: AtomicU16::new(0),
		}
	}

	fn capacity(&self) -> u16 {
		self.slots.len() as u16
	}

	fn len(&self) -> u16 {
		self.slots
			.iter()
			.filter(|slot| !slot.free.load(Ordering::SeqCst))
			.count() as u16
	}

	fn try_reserve(&self) -> Result<Key, ArenaFull> {
		loop {
			let first_free_slot_index = self.first_free_slot_index.load(Ordering::SeqCst);
			if first_free_slot_index == NO_NEXT_FREE_SLOT {
				return Err(ArenaFull);
			}
			let slot = &self.slots[first_free_slot_index as usize];
			if self
				.first_free_slot_index
				.compare_exchange_weak(
					first_free_slot_index,
					slot.next_free_slot_index.load(Ordering::SeqCst),
					Ordering::SeqCst,
					Ordering::SeqCst,
				)
				.is_ok()
			{
				slot.free.store(false, Ordering::SeqCst);
				return Ok(Key {
					index: first_free_slot_index,
					generation: slot.generation.load(Ordering::SeqCst),
				});
			}
		}
	}

	fn free(&self, index: u16) {
		let slot = &self.slots[index as usize];
		slot.free.store(true, Ordering::SeqCst);
		slot.generation.fetch_add(1, Ordering::SeqCst);
		loop {
			let first_free_slot_index = self.first_free_slot_index.load(Ordering::SeqCst);
			slot.next_free_slot_index
				.store(first_free_slot_index, Ordering::SeqCst);
			if self
				.first_free_slot_index
				.compare_exchange_weak(
					first_free_slot_index,
					index,
					Ordering::SeqCst,
					Ordering::SeqCst,
				)
				.is_ok()
			{
				break;
			}
		}
	}
}

/// Manages [`Key`] reservations for an [`Arena`](super::Arena).
#[derive(Debug, Clone)]
pub struct Controller(Arc<ControllerInner>);

impl Controller {
	pub(crate) fn new(capacity: u16) -> Self {
		Self(Arc::new(ControllerInner::new(capacity)))
	}

	/// Returns the total capacity of the arena.
	pub fn capacity(&self) -> u16 {
		self.0.capacity()
	}

	/// Returns the number of items in the arena.
	pub fn len(&self) -> u16 {
		self.0.len()
	}

	/// Returns `true` if the arena is empty.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Tries to reserve a key for the [`Arena`](super::Arena).
	pub fn try_reserve(&self) -> Result<Key, ArenaFull> {
		self.0.try_reserve()
	}

	pub(crate) fn free(&self, index: u16) {
		self.0.free(index);
	}
}
