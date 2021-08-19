//! A user-controllable timing source for instances and tweens.

mod handle;

pub use handle::*;

use std::sync::{
	atomic::{AtomicBool, AtomicU64, Ordering},
	Arc,
};

use atomic_arena::Index;

use crate::{
	manager::resources::Parameters,
	value::{cached::CachedValue, Value},
};

/// A unique identifier for a clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockId(pub(crate) Index);

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

pub(crate) struct Clock {
	shared: Arc<ClockShared>,
	ticking: bool,
	interval: CachedValue,
	ticks: u64,
	tick_timer: f64,
}

impl Clock {
	pub fn new(interval: Value) -> Self {
		Self {
			shared: Arc::new(ClockShared::new()),
			ticking: false,
			interval: CachedValue::new(0.0.., interval, 1.0),
			ticks: 0,
			tick_timer: 1.0,
		}
	}

	pub fn shared(&self) -> Arc<ClockShared> {
		self.shared.clone()
	}

	pub fn ticking(&self) -> bool {
		self.ticking
	}

	pub fn ticks(&self) -> u64 {
		self.ticks
	}

	pub fn set_interval(&mut self, interval: Value) {
		self.interval.set(interval);
	}

	pub fn start(&mut self) {
		self.ticking = true;
		self.shared.ticking.store(true, Ordering::SeqCst);
	}

	pub fn pause(&mut self) {
		self.ticking = false;
		self.shared.ticking.store(false, Ordering::SeqCst);
	}

	pub fn stop(&mut self) {
		self.pause();
		self.ticks = 0;
		self.shared.ticks.store(0, Ordering::SeqCst);
	}

	pub fn update(&mut self, dt: f64, parameters: &Parameters) {
		self.interval.update(parameters);
		if self.ticking {
			self.tick_timer -= dt / self.interval.get();
			while self.tick_timer <= 0.0 {
				self.tick_timer += 1.0;
				self.ticks += 1;
				self.shared.ticks.fetch_add(1, Ordering::SeqCst);
			}
		}
	}
}
