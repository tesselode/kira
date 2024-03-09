use std::sync::Arc;

use crate::{
	command::ValueChangeCommand,
	tween::{Tween, Value},
};

use super::{ClockId, ClockShared, ClockSpeed, ClockTime, CommandWriters};

/// Controls a clock.
///
/// When a [`ClockHandle`] is dropped, the corresponding clock
/// will be removed.
pub struct ClockHandle {
	pub(crate) id: ClockId,
	pub(crate) shared: Arc<ClockShared>,
	pub(crate) command_writers: CommandWriters,
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

	/// Returns the time between ticks (from `0.0` to `1.0`) of the clock.
	///
	/// A time of `0.5` is halfway between two ticks.
	pub fn fractional_position(&self) -> f64 {
		self.shared.fractional_position()
	}

	/// Sets the speed of the clock.
	pub fn set_speed(&mut self, speed: impl Into<Value<ClockSpeed>>, tween: Tween) {
		self.command_writers.speed_change.write(ValueChangeCommand {
			target: speed.into(),
			tween,
		})
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
		self.command_writers.reset.write(());
		self.command_writers.set_ticking.write(false);
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
