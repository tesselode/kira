pub mod static_sound;

use crate::frame::Frame;

use super::metadata::SoundMetadata;

pub trait SoundData: Send + Sync {
	fn duration(&self) -> f64;

	fn frame_at_position(&self, position: f64) -> Frame;

	fn metadata(&self) -> SoundMetadata {
		SoundMetadata::default()
	}
}
