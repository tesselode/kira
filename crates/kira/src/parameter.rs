//! Tweenable values for controlling settings.

mod handle;
mod parameters;

pub use handle::*;
pub use parameters::*;

use std::sync::{
	atomic::{AtomicBool, AtomicU64, Ordering},
	Arc,
};

use atomic_arena::Key;

use crate::{
	clock::Clocks,
	tween::{Tween, Tweenable},
};

type JustFinishedTween = bool;

/// A unique identifier for a parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParameterId(pub(crate) Key);

pub(crate) struct ParameterShared {
	value: AtomicU64,
	paused: AtomicBool,
	removed: AtomicBool,
}

impl ParameterShared {
	pub fn new(value: f64) -> Self {
		Self {
			value: AtomicU64::new(value.to_bits()),
			paused: AtomicBool::new(false),
			removed: AtomicBool::new(false),
		}
	}

	pub fn value(&self) -> f64 {
		f64::from_bits(self.value.load(Ordering::SeqCst))
	}

	pub fn paused(&self) -> bool {
		self.paused.load(Ordering::SeqCst)
	}

	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

pub(crate) struct Parameter {
	tweenable: Tweenable,
	paused: bool,
	shared: Arc<ParameterShared>,
}

impl Parameter {
	pub fn new(initial_value: f64) -> Self {
		Self {
			tweenable: Tweenable::new(initial_value),
			paused: false,
			shared: Arc::new(ParameterShared::new(initial_value)),
		}
	}

	pub(crate) fn shared(&self) -> Arc<ParameterShared> {
		self.shared.clone()
	}

	pub fn value(&self) -> f64 {
		self.tweenable.value()
	}

	pub fn pause(&mut self) {
		self.paused = true;
		self.shared.paused.store(true, Ordering::SeqCst);
	}

	pub fn resume(&mut self) {
		self.paused = false;
		self.shared.paused.store(false, Ordering::SeqCst);
	}

	pub fn set(&mut self, target: f64, tween: Tween) {
		self.tweenable.set(target, tween);
	}

	pub(crate) fn on_start_processing(&self) {
		self.shared
			.value
			.store(self.tweenable.value().to_bits(), Ordering::SeqCst);
	}

	pub fn update(&mut self, dt: f64, clocks: &Clocks) -> JustFinishedTween {
		if self.paused {
			return false;
		}
		self.tweenable.update(dt, clocks)
	}
}
