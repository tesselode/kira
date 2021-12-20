use ringbuf::Producer;

use crate::{tween::Tween, CommandError};

use super::{Command, FilterMode};

pub struct FilterHandle {
	pub(super) command_producer: Producer<Command>,
}

impl FilterHandle {
	pub fn set_mode(&mut self, mode: FilterMode) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetMode(mode))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	pub fn set_cutoff(&mut self, cutoff: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetCutoff(cutoff, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	pub fn set_resonance(&mut self, resonance: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetResonance(resonance, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	pub fn set_mix(&mut self, mix: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetMix(mix, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
