#[cfg(test)]
mod test;

use std::{sync::Arc, time::Duration};

use ringbuf::RingBuffer;

use crate::{
	dsp::Frame,
	sound::{Sound, SoundData},
};

use super::{handle::StaticSoundHandle, sound::StaticSound, StaticSoundSettings};

const COMMAND_BUFFER_CAPACITY: usize = 8;

/// A piece of audio loaded into memory all at once.
///
/// These can be cheaply cloned, as the audio data is shared
/// among all clones.
#[derive(Clone)]
pub struct StaticSoundData {
	/// The sample rate of the audio (in Hz).
	pub sample_rate: u32,
	/// The raw samples that make up the audio.
	pub frames: Arc<Vec<Frame>>,
	/// Settings for the sound.
	pub settings: StaticSoundSettings,
}

impl StaticSoundData {
	/// Returns the duration of the audio.
	pub fn duration(&self) -> Duration {
		Duration::from_secs_f64(self.frames.len() as f64 / self.sample_rate as f64)
	}

	pub(super) fn split(self) -> (StaticSound, StaticSoundHandle) {
		let (command_producer, command_consumer) = RingBuffer::new(COMMAND_BUFFER_CAPACITY).split();
		let sound = StaticSound::new(self, command_consumer);
		let shared = sound.shared();
		(
			sound,
			StaticSoundHandle {
				command_producer,
				shared,
			},
		)
	}
}

impl SoundData for StaticSoundData {
	type Error = ();

	type Handle = StaticSoundHandle;

	#[allow(clippy::type_complexity)]
	fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
		let (sound, handle) = self.split();
		Ok((Box::new(sound), handle))
	}
}
