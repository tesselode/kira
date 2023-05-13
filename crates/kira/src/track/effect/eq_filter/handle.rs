use ringbuf::HeapProducer;

use crate::{
	tween::{Tween, Value},
	CommandError,
};

use super::Command;

/// Controls an EQ filter.
pub struct EqFilterHandle {
	pub(super) command_producer: HeapProducer<Command>,
}

impl EqFilterHandle {
	/// Sets the "center" or "corner" of the frequency range to adjust in Hz
	/// (for bell or shelf curves, respectively).
	pub fn set_frequency(
		&mut self,
		frequency: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetFrequency(frequency.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the volume adjustment for frequencies in the specified range (in decibels).
	pub fn set_gain(
		&mut self,
		gain: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetGain(gain.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the width of the frequency range to adjust.
	///
	/// A higher Q value results in a narrower range of frequencies being adjusted.
	/// The value should be greater than `0.0`.
	pub fn set_q(&mut self, q: impl Into<Value<f64>>, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetQ(q.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
