use std::sync::Arc;

use atomic::{Atomic, Ordering};
use ringbuf::Producer;

use crate::error::CommandQueueFullError;

use super::Command;

pub struct InstanceHandle {
	public_playback_position: Arc<Atomic<f64>>,
	command_producer: Producer<Command>,
}

impl InstanceHandle {
	pub(crate) fn new(
		public_playback_position: Arc<Atomic<f64>>,
		command_producer: Producer<Command>,
	) -> Self {
		Self {
			public_playback_position,
			command_producer,
		}
	}

	pub fn playback_position(&self) -> f64 {
		self.public_playback_position.load(Ordering::Relaxed)
	}

	pub fn pause(&mut self) -> Result<(), CommandQueueFullError> {
		self.command_producer
			.push(Command::Pause)
			.map_err(|_| CommandQueueFullError)
	}

	pub fn resume(&mut self) -> Result<(), CommandQueueFullError> {
		self.command_producer
			.push(Command::Resume)
			.map_err(|_| CommandQueueFullError)
	}

	pub fn stop(&mut self) -> Result<(), CommandQueueFullError> {
		self.command_producer
			.push(Command::Stop)
			.map_err(|_| CommandQueueFullError)
	}
}
