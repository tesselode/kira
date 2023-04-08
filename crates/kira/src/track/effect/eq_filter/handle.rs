use ringbuf::HeapProducer;

use crate::{parameter::Value, tween::Tween, CommandError};

use super::Command;

pub struct EqFilterHandle {
	pub(super) command_producer: HeapProducer<Command>,
}

impl EqFilterHandle {
	/// Sets the frequency of the filter (in hertz).
	pub fn set_frequency(
		&mut self,
		frequency: impl Into<Value<f32>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetFrequency(frequency.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the gain of the filter (in hertz).
	pub fn set_gain(
		&mut self,
		gain: impl Into<Value<f32>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetGain(gain.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the q value of the filter (in hertz).
	pub fn set_q(&mut self, q: impl Into<Value<f32>>, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetQ(q.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
