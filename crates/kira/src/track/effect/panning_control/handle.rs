use ringbuf::HeapProducer;

use crate::{
	tween::{Tween, Value},
	CommandError,
};

use super::Command;

/// Controls a panning control effect.
pub struct PanningControlHandle {
	pub(super) command_producer: HeapProducer<Command>,
}

impl PanningControlHandle {
	/// Sets the panning adjustment to apply to input audio.
	pub fn set_panning(
		&mut self,
		panning: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetPanning(panning.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
