use std::{error::Error, fmt::Display};

/// An error that's returned when trying to change the volume of a track route
/// that did not exist originally.
#[derive(Debug)]
pub struct NonexistentRoute;

impl Display for NonexistentRoute {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("Cannot change the volume of a track route that did not exist originally")
	}
}

impl Error for NonexistentRoute {}
