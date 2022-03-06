use ringbuf::Producer;

use crate::{tween::Tween, CommandError};

use super::Command;

/// Controls a panning control effect.
pub struct PanningControlHandle {
	pub(super) command_producer: Producer<Command>,
}

impl PanningControlHandle {
	/// Sets the panning adjustment to apply to input audio.
	pub fn set_panning(&mut self, panning: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetPanning(panning, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
