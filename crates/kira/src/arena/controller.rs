use std::sync::{
	atomic::{AtomicBool, AtomicUsize, Ordering},
	Arc,
};

use crate::arena::{ArenaFull, Key};

/// Represents that a [`ControllerSlot`] does not have a free slot
/// after it.
///
/// This is used because the next free slot variable is an
/// [`AtomicUsize`], but we still need some way to represent the
/// absence of a next free slot.
const NO_NEXT_FREE_SLOT: usize = usize::MAX;

#[derive(Debug)]
struct ControllerSlot {
	free: AtomicBool,
	generation: AtomicUsize,
	next_free_slot_index: AtomicUsize,
}

/// The shared state for all [`Controller`]s for an [`Arena`](super::Arena).
#[derive(Debug)]
struct ControllerInner {
	slots: Vec<ControllerSlot>,
	first_free_slot_index: AtomicUsize,
}

impl ControllerInner {
	fn new(capacity: usize) -> Self {
		Self {
			slots: (0..capacity)
				.map(|i| ControllerSlot {
					free: AtomicBool::new(true),
					generation: AtomicUsize::new(0),
					next_free_slot_index: AtomicUsize::new(if i < capacity - 1 {
						i + 1
					} else {
						NO_NEXT_FREE_SLOT
					}),
				})
				.collect(),
			first_free_slot_index: AtomicUsize::new(0),
		}
	}

	fn capacity(&self) -> usize {
		self.slots.len()
	}

	fn len(&self) -> usize {
		self.slots
			.iter()
			.filter(|slot| !slot.free.load(Ordering::SeqCst))
			.count()
	}

	fn try_reserve(&self) -> Result<Key, ArenaFull> {
		loop {
			let first_free_slot_index = self.first_free_slot_index.load(Ordering::SeqCst);
			if first_free_slot_index == NO_NEXT_FREE_SLOT {
				return Err(ArenaFull);
			}
			let slot = &self.slots[first_free_slot_index];
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

	fn free(&self, index: usize) {
		let slot = &self.slots[index];
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
	pub(crate) fn new(capacity: usize) -> Self {
		Self(Arc::new(ControllerInner::new(capacity)))
	}

	/// Returns the total capacity of the arena.
	pub fn capacity(&self) -> usize {
		self.0.capacity()
	}

	/// Returns the number of items in the arena.
	pub fn len(&self) -> usize {
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

	pub(crate) fn free(&self, index: usize) {
		self.0.free(index);
	}
}
