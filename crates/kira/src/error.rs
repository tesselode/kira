use std::{error::Error, fmt::Display};

/// An error that is returned when a resource cannot be added because the
/// maximum capacity for that resource has been reached.
///
/// You can adjust these capacities using [`Capacities`](crate::manager::Capacities).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceLimitReached;

impl Display for ResourceLimitReached {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("Could not add a resource because the maximum capacity for that resource has been reached")
	}
}

impl Error for ResourceLimitReached {}
