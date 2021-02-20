use std::ops::RangeBounds;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("Cannot push to the StaticVec because it is full")]
pub struct StaticVecFullError;

/// A thin wrapper around `Vec`s that prevents any memory
/// allocation from occurring after the `Vec` is created.
///
/// This is used to avoid accidental memory allocation
/// on the audio thread, which can theoretically take
/// an indefinite amount of time and lead to audio glitches.
pub struct StaticVec<T> {
	vec: Vec<T>,
}

impl<T> StaticVec<T> {
	pub fn new(capacity: usize) -> Self {
		Self {
			vec: Vec::with_capacity(capacity),
		}
	}

	pub fn len(&self) -> usize {
		self.vec.len()
	}

	pub fn capacity(&self) -> usize {
		self.vec.capacity()
	}

	pub fn get(&self, index: usize) -> Option<&T> {
		self.vec.get(index)
	}

	pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
		self.vec.get_mut(index)
	}

	pub fn iter(&self) -> std::slice::Iter<T> {
		self.vec.iter()
	}

	pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
		self.vec.iter_mut()
	}

	/// Tries to push a value to the `Vec`.
	///
	/// - If the `Vec` is full, returns `Err(StaticVecFullError)`
	/// - Otherwise, returns `Ok(())`
	///
	/// As of the time of writing, I never actually check the `Result`,
	/// instead opting to ignore it with `ok()`. Most resources are
	/// managed with the `basedrop` crate, which will automatically move
	/// dropped items to the main thread for collection, so I can discard
	/// any value in the `Result` without worrying about triggering
	/// any memory deallocation on the audio thread.
	///
	/// That being said, I still think it's best to return a `Result` that
	/// indicates if the value was added successfully. That way,
	/// clippy will encourage me to make an intentional decision about
	/// what to do with the `Result`. If I move the code that produces
	/// the user-facing out-of-capacity error to the audio thread in the
	/// future, this will come in handy in that situation as well.
	pub fn try_push(&mut self, value: T) -> Result<(), StaticVecFullError> {
		if self.len() > self.capacity() {
			return Err(StaticVecFullError);
		}
		self.vec.push(value);
		Ok(())
	}

	pub fn drain(&mut self, range: impl RangeBounds<usize>) -> std::vec::Drain<T> {
		self.vec.drain(range)
	}
}

impl<'a, T> IntoIterator for &'a StaticVec<T> {
	type Item = &'a T;

	type IntoIter = std::slice::Iter<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T> IntoIterator for &'a mut StaticVec<T> {
	type Item = &'a mut T;

	type IntoIter = std::slice::IterMut<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}
