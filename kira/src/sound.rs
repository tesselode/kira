mod handle;
pub mod instance;
mod seamless_loop;
pub mod static_sound;
pub(crate) mod wrapper;

pub use handle::*;
pub use seamless_loop::*;

use std::time::Duration;

use atomic_arena::Index;

use crate::Frame;

/// A unique identifier for a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundId(pub(crate) Index);

/// Represents a finite piece of audio.
pub trait Sound: Send + Sync {
	/// Returns the duration of the sound.
	fn duration(&self) -> Duration;

	/// Returns the [`Frame`] that the sound should output
	/// at a given playback position.
	fn frame_at_position(&self, position: f64) -> Frame;

	fn default_loop_start(&self) -> Option<f64> {
		None
	}
}
