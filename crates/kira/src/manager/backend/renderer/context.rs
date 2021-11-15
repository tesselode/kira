use std::sync::atomic::{AtomicU8, Ordering};

use crate::manager::MainPlaybackState;

pub(crate) struct Context {
	pub(super) sample_rate: u32,
	pub(super) dt: f64,
	pub(super) state: AtomicU8,
}

impl Context {
	pub fn new(sample_rate: u32) -> Self {
		Self {
			sample_rate,
			dt: 1.0 / sample_rate as f64,
			state: AtomicU8::new(MainPlaybackState::Playing as u8),
		}
	}

	pub fn sample_rate(&self) -> u32 {
		self.sample_rate
	}

	pub fn state(&self) -> MainPlaybackState {
		MainPlaybackState::from_u8(self.state.load(Ordering::SeqCst))
	}
}
