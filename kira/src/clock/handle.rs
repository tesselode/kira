use std::sync::Arc;

use crate::{
	error::CommandError,
	manager::command::{producer::CommandProducer, ClockCommand, Command},
	value::Value,
};

use super::{ClockId, ClockShared, ClockTime};

/// Controls a clock.
///
/// When a [`ClockHandle`] is dropped, the corresponding clock
/// will be removed.
pub struct ClockHandle {
	pub(crate) id: ClockId,
	pub(crate) shared: Arc<ClockShared>,
	pub(crate) command_producer: CommandProducer,
}

impl ClockHandle {
	/// Returns the unique identifier for the clock.
	pub fn id(&self) -> ClockId {
		self.id
	}

	/// Returns `true` if the clock is currently ticking
	/// and `false` if not.
	pub fn ticking(&self) -> bool {
		self.shared.ticking()
	}

	/// Returns the current time of the clock.
	pub fn time(&self) -> ClockTime {
		ClockTime {
			clock: self.id,
			ticks: self.shared.ticks(),
		}
	}

	/// Sets the duration of time between each tick (in seconds).
	pub fn set_interval(&mut self, interval: impl Into<Value>) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Clock(ClockCommand::SetInterval(
				self.id,
				interval.into(),
			)))
	}

	/// Starts or resumes the clock.
	pub fn start(&mut self) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Clock(ClockCommand::Start(self.id)))
	}

	/// Pauses the clock.
	pub fn pause(&mut self) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Clock(ClockCommand::Pause(self.id)))
	}

	/// Stops and resets the clock.
	pub fn stop(&mut self) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Clock(ClockCommand::Stop(self.id)))
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
