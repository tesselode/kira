pub mod static_sound;

use crate::frame::Frame;

pub trait SoundData: Send + Sync {
	fn duration(&self) -> f64;

	fn frame_at_position(&self, position: f64) -> Frame;
}
