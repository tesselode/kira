use ringbuf::Producer;

use crate::{tween::Tween, CommandError};

use super::Command;

pub struct DelayHandle {
	pub(super) command_producer: Producer<Command>,
}

impl DelayHandle {
	pub fn set_delay_time(&mut self, delay_time: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetDelayTime(delay_time, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	pub fn set_feedback(&mut self, feedback: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetFeedback(feedback, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	pub fn set_mix(&mut self, mix: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetMix(mix, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
