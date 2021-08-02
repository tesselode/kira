mod seamless_loop;
pub mod static_sound;

pub use seamless_loop::*;

use std::time::Duration;

use crate::frame::Frame;

pub trait SoundData: Send + Sync {
	fn duration(&self) -> Duration;

	fn frame_at_position(&self, position: f64) -> Frame;

	fn default_loop_start(&self) -> Option<f64> {
		None
	}
}
