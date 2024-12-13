use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Debug)]
pub(crate) struct ClockShared {
	pub ticking: AtomicBool,
	pub ticks: AtomicU64,
	pub fractional_position: AtomicU64,
	pub removed: AtomicBool,
}

impl ClockShared {
	#[must_use]
	pub fn new() -> Self {
		Self {
			ticking: AtomicBool::new(false),
			ticks: AtomicU64::new(0),
			fractional_position: AtomicU64::new(0.0f64.to_bits()),
			removed: AtomicBool::new(false),
		}
	}

	#[must_use]
	pub fn ticking(&self) -> bool {
		self.ticking.load(Ordering::SeqCst)
	}

	#[must_use]
	pub fn ticks(&self) -> u64 {
		self.ticks.load(Ordering::SeqCst)
	}

	#[must_use]
	pub fn fractional_position(&self) -> f64 {
		f64::from_bits(self.fractional_position.load(Ordering::SeqCst))
	}

	#[must_use]
	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}
