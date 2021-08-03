mod seamless_loop;
pub mod static_sound;

pub use seamless_loop::*;

use std::time::Duration;

use crate::frame::Frame;

/// Provides the underlying data for a sound.
pub trait SoundData: Send + Sync {
	/// Returns the duration of the sound.
	fn duration(&self) -> Duration;

	/// Returns the [`Frame`] that a sound should output
	/// at a given playback position.
	fn frame_at_position(&self, position: f64) -> Frame;

	fn default_loop_start(&self) -> Option<f64> {
		None
	}
}
