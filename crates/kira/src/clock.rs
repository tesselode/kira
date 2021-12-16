//! Precise timing for audio events.

mod clocks;
mod handle;
mod time;

pub use clocks::*;
pub use handle::*;
pub use time::*;

use std::sync::{
	atomic::{AtomicBool, AtomicU64, Ordering},
	Arc,
};

use atomic_arena::Key;

use crate::{
	parameter::Parameters,
	value::{CachedValue, Value},
};

/// A unique identifier for a [`Clock`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockId(pub(crate) Key);

pub(crate) struct ClockShared {
	ticking: AtomicBool,
	ticks: AtomicU64,
	removed: AtomicBool,
}

impl ClockShared {
	pub fn new() -> Self {
		Self {
			ticking: AtomicBool::new(false),
			ticks: AtomicU64::new(0),
			removed: AtomicBool::new(false),
		}
	}

	pub fn ticking(&self) -> bool {
		self.ticking.load(Ordering::SeqCst)
	}

	pub fn ticks(&self) -> u64 {
		self.ticks.load(Ordering::SeqCst)
	}

	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

/// A user-controllable timing source.
///
/// You will only need to interact with [`Clock`]s directly
/// if you're writing your own [`Sound`](crate::sound::Sound)s.
/// Otherwise, you'll be interacting with clocks using a
/// [`ClockHandle`].
pub struct Clock {
	shared: Arc<ClockShared>,
	ticking: bool,
	interval: CachedValue,
	ticks: u64,
	tick_timer: f64,
}

impl Clock {
	pub(crate) fn new(interval: Value) -> Self {
		Self {
			shared: Arc::new(ClockShared::new()),
			ticking: false,
			interval: CachedValue::new(0.0.., interval, 1.0),
			ticks: 0,
			tick_timer: 1.0,
		}
	}

	pub(crate) fn shared(&self) -> Arc<ClockShared> {
		self.shared.clone()
	}

	/// Returns `true` if the clock is currently running.
	pub fn ticking(&self) -> bool {
		self.ticking
	}

	/// Returns the number of times the clock has ticked.
	pub fn ticks(&self) -> u64 {
		self.ticks
	}

	pub(crate) fn set_interval(&mut self, interval: Value) {
		self.interval.set(interval);
	}

	pub(crate) fn start(&mut self) {
		self.ticking = true;
		self.shared.ticking.store(true, Ordering::SeqCst);
	}

	pub(crate) fn pause(&mut self) {
		self.ticking = false;
		self.shared.ticking.store(false, Ordering::SeqCst);
	}

	pub(crate) fn stop(&mut self) {
		self.pause();
		self.ticks = 0;
		self.shared.ticks.store(0, Ordering::SeqCst);
	}

	pub(crate) fn update(&mut self, dt: f64, parameters: &Parameters) -> Option<u64> {
		let mut ticked = false;
		self.interval.update(parameters);
		if self.ticking {
			self.tick_timer -= dt / self.interval.get();
			while self.tick_timer <= 0.0 {
				self.tick_timer += 1.0;
				self.ticks += 1;
				self.shared.ticks.fetch_add(1, Ordering::SeqCst);
				ticked = true;
			}
		}
		if ticked {
			Some(self.ticks)
		} else {
			None
		}
	}
}
