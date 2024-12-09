//! Error types.

use std::{error::Error, fmt::Display};

/// Returned when trying to reserve an key on a
/// full [`Arena`](super::Arena).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaFull;

impl Display for ArenaFull {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("Cannot reserve an key because the arena is full")
	}
}

impl Error for ArenaFull {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// An error that can occur when inserting an item
/// into an [`Arena`](super::Arena) with an existing
/// [`Key`](super::Key).
pub enum InsertWithKeyError {
	/// Cannot insert with this key because it is not reserved.
	KeyNotReserved,
	/// Cannot insert with this key because the slot index
	/// or generation is invalid for this arena.
	InvalidKey,
}

impl Display for InsertWithKeyError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			InsertWithKeyError::KeyNotReserved => f.write_str("Cannot insert with this key because it is not reserved"),
			InsertWithKeyError::InvalidKey => f.write_str("Cannot insert with this key because the slot index or generation is invalid for this arena."),
		}
	}
}

impl Error for InsertWithKeyError {}
