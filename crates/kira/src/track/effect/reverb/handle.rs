use ringbuf::Producer;

use crate::{tween::Tween, CommandError};

use super::Command;

pub struct ReverbHandle {
	pub(super) command_producer: Producer<Command>,
}

impl ReverbHandle {
	pub fn set_feedback(&mut self, feedback: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetFeedback(feedback, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	pub fn set_damping(&mut self, damping: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetDamping(damping, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	pub fn set_stereo_width(
		&mut self,
		stereo_width: f64,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetStereoWidth(stereo_width, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	pub fn set_mix(&mut self, mix: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetMix(mix, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
