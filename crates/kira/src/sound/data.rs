mod static_sound;

pub use static_sound::*;

use crate::dsp::Frame;

#[allow(clippy::len_without_is_empty)]
pub trait SoundData: Send {
	fn sample_rate(&mut self) -> u32;

	fn len(&mut self) -> usize;

	fn frame(&mut self, index: usize) -> Frame;
}
