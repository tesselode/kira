use std::sync::atomic::{AtomicU8, Ordering};

use crate::manager::MainPlaybackState;

pub(crate) struct RendererShared {
	pub(super) state: AtomicU8,
}

impl RendererShared {
	pub fn new() -> Self {
		Self {
			state: AtomicU8::new(MainPlaybackState::Playing as u8),
		}
	}

	pub fn state(&self) -> MainPlaybackState {
		MainPlaybackState::from_u8(self.state.load(Ordering::SeqCst))
	}
}
