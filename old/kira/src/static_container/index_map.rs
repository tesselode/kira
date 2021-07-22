use std::hash::Hash;

use indexmap::IndexMap;

/// A thin wrapper around `IndexMap`s that prevents any memory
/// allocation from occurring after the `IndexMap` is created.
///
/// This is used to avoid accidental memory allocation
/// on the audio thread, which can theoretically take
/// an indefinite amount of time and lead to audio glitches.
#[derive(Debug, Clone)]
pub struct StaticIndexMap<K: Eq + Hash, V> {
	capacity: usize,
	index_map: IndexMap<K, V>,
}

impl<K: Eq + Hash, V> StaticIndexMap<K, V> {
	pub fn new(capacity: usize) -> Self {
		Self {
			capacity,
			// The IndexMap is initialized with twice the requested
			// capacity to make sure the map will never need to allocate
			// memory to maintain the requested capacity.
			// See here: https://github.com/rust-lang/hashbrown/pull/255
			index_map: IndexMap::with_capacity(capacity * 2),
		}
	}

	pub fn len(&self) -> usize {
		self.index_map.len()
	}

	pub fn capacity(&self) -> usize {
		self.capacity
	}

	pub fn get(&self, key: &K) -> Option<&V> {
		self.index_map.get(key)
	}

	pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
		self.index_map.get_index(index)
	}

	pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
		self.index_map.get_mut(key)
	}

	pub fn iter(&self) -> indexmap::map::Iter<K, V> {
		self.index_map.iter()
	}

	pub fn iter_mut(&mut self) -> indexmap::map::IterMut<K, V> {
		self.index_map.iter_mut()
	}

	pub fn values_mut(&mut self) -> indexmap::map::ValuesMut<K, V> {
		self.index_map.values_mut()
	}

	/// Tries to add a key value pair to the map.
	///
	/// - If the map is full, returns the key and value back through
	/// an `Err`
	/// - Otherwise, returns `Ok` with whatever item previously had
	/// that ID, if any
	///
	/// As of the time of writing, I never actually check the `Result`,
	/// instead opting to ignore it with `ok()`. Most resources are
	/// managed with the `basedrop` crate, which will automatically move
	/// dropped items to the main thread for collection, so I can discard
	/// any value in the `Result` without worrying about triggering
	/// any memory deallocation on the audio thread.
	///
	/// That being said, I still think it's best to return a `Result` that
	/// indicates if the key value pair was added successfully. That way,
	/// clippy will encourage me to make an intentional decision about
	/// what to do with the `Result`. If I move the code that produces
	/// the user-facing out-of-capacity error to the audio thread in the
	/// future, this will come in handy in that situation as well.
	pub fn try_insert(&mut self, key: K, value: V) -> Result<Option<V>, (K, V)> {
		if self.len() >= self.capacity() {
			return Err((key, value));
		}
		Ok(self.index_map.insert(key, value))
	}

	pub fn remove(&mut self, key: &K) -> Option<V> {
		self.index_map.remove(key)
	}

	pub fn shift_remove(&mut self, key: &K) -> Option<V> {
		self.index_map.shift_remove(key)
	}

	pub fn shift_remove_index(&mut self, index: usize) -> Option<(K, V)> {
		self.index_map.shift_remove_index(index)
	}
}

impl<'a, K: Eq + Hash, V> IntoIterator for &'a StaticIndexMap<K, V> {
	type Item = (&'a K, &'a V);

	type IntoIter = indexmap::map::Iter<'a, K, V>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, K: Eq + Hash, V> IntoIterator for &'a mut StaticIndexMap<K, V> {
	type Item = (&'a K, &'a mut V);

	type IntoIter = indexmap::map::IterMut<'a, K, V>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}
