use std::sync::{atomic::Ordering, Arc};

use ringbuf::HeapProducer;

use crate::{modulator::ModulatorId, tween::Tween, CommandError};

use super::{command::Command, TweenerShared};

/// Controls a tweener.
pub struct TweenerHandle {
	pub(super) id: ModulatorId,
	pub(super) command_producer: HeapProducer<Command>,
	pub(super) shared: Arc<TweenerShared>,
}

impl TweenerHandle {
	/// Returns the unique identifier for the modulator.
	pub fn id(&self) -> ModulatorId {
		self.id
	}

	/// Starts a transition from the current value to a target value with
	/// the given tween.
	pub fn set(&mut self, target: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Set { target, tween })
			.map_err(|_| CommandError::CommandQueueFull)
	}
}

impl From<&TweenerHandle> for ModulatorId {
	fn from(handle: &TweenerHandle) -> Self {
		handle.id
	}
}

impl Drop for TweenerHandle {
	fn drop(&mut self) {
		self.shared.removed.store(true, Ordering::SeqCst);
	}
}
