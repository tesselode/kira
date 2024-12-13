use std::sync::{atomic::Ordering, Arc};

use crate::{modulator::ModulatorId, tween::Tween};

use super::{CommandWriters, TweenerShared};

/// Controls a tweener.
#[derive(Debug)]
pub struct TweenerHandle {
	pub(super) id: ModulatorId,
	pub(super) command_writers: CommandWriters,
	pub(super) shared: Arc<TweenerShared>,
}

impl TweenerHandle {
	/// Returns the unique identifier for the modulator.
	pub fn id(&self) -> ModulatorId {
		self.id
	}

	/// Starts a transition from the current value to a target value with
	/// the given tween.
	pub fn set(&mut self, target: f64, tween: Tween) {
		self.command_writers.set.write((target, tween))
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
