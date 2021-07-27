use std::sync::atomic::{AtomicU64, Ordering};

pub struct Context {
	pub(super) sample_count: AtomicU64,
	pub(super) sample_rate: u32,
	pub(super) dt: f64,
}

impl Context {
	pub fn new(sample_rate: u32) -> Self {
		Self {
			sample_count: AtomicU64::new(0),
			sample_rate,
			dt: 1.0 / sample_rate as f64,
		}
	}

	pub fn sample_count(&self) -> u64 {
		self.sample_count.load(Ordering::SeqCst)
	}

	pub fn sample_rate(&self) -> u32 {
		self.sample_rate
	}
}
