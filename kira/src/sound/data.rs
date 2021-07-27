pub mod static_sound;

use std::time::Duration;

use crate::frame::Frame;

use super::metadata::SoundMetadata;

pub trait SoundData: Send + Sync {
	fn duration(&self) -> Duration;

	fn frame_at_position(&self, position: f64) -> Frame;

	fn metadata(&self) -> SoundMetadata {
		SoundMetadata::default()
	}
}
