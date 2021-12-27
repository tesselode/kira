use ringbuf::Producer;

use crate::{tween::Tween, CommandError};

use super::{Command, DistortionKind};

pub struct DistortionHandle {
	pub(super) command_producer: Producer<Command>,
}

impl DistortionHandle {
	pub fn set_kind(&mut self, kind: DistortionKind) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetKind(kind))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	pub fn set_drive(&mut self, drive: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetDrive(drive, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	pub fn set_mix(&mut self, mix: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetMix(mix, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
