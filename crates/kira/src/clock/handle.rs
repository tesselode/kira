use std::sync::Arc;

use crate::command::handle_param_setters;

use super::{ClockId, ClockShared, ClockSpeed, ClockTime, CommandWriters};

/// Controls a clock.
///
/// When a [`ClockHandle`] is dropped, the corresponding clock
/// will be removed.
#[derive(Debug)]
pub struct ClockHandle {
	pub(crate) id: ClockId,
	pub(crate) shared: Arc<ClockShared>,
	pub(crate) command_writers: CommandWriters,
}

impl ClockHandle {
	/// Returns the unique identifier for the clock.
	#[must_use]
	pub fn id(&self) -> ClockId {
		self.id
	}

	/// Returns `true` if the clock is currently ticking
	/// and `false` if not.
	#[must_use]
	pub fn ticking(&self) -> bool {
		self.shared.ticking()
	}

	/// Returns the current time of the clock.
	#[must_use]
	pub fn time(&self) -> ClockTime {
		ClockTime {
			clock: self.id,
			ticks: self.shared.ticks(),
			fraction: self.shared.fractional_position(),
		}
	}

	handle_param_setters! {
		/// Sets the speed of the clock.
		speed: ClockSpeed,
	}

	/// Starts or resumes the clock.
	pub fn start(&mut self) {
		self.command_writers.set_ticking.write(true)
	}

	/// Pauses the clock.
	pub fn pause(&mut self) {
		self.command_writers.set_ticking.write(false)
	}

	/// Stops and resets the clock.
	pub fn stop(&mut self) {
		self.command_writers.set_ticking.write(false);
		self.command_writers.reset.write(());
	}
}

impl Drop for ClockHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}

impl From<&ClockHandle> for ClockId {
	fn from(handle: &ClockHandle) -> Self {
		handle.id()
	}
}
