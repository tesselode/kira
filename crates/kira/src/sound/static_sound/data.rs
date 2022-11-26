#[cfg(feature = "symphonia")]
mod from_file;

#[cfg(test)]
mod test;

use std::{
	fmt::{Debug, Formatter},
	sync::Arc,
	time::Duration,
};

use ringbuf::HeapRb;

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
#[derive(Clone, PartialEq)]
pub struct StaticSoundData {
	/// The sample rate of the audio (in Hz).
	pub sample_rate: u32,
	/// The raw samples that make up the audio.
	pub frames: Arc<[Frame]>,
	/// Settings for the sound.
	pub settings: StaticSoundSettings,
}

impl StaticSoundData {
	/// Returns the duration of the audio.
	pub fn duration(&self) -> Duration {
		Duration::from_secs_f64(self.frames.len() as f64 / self.sample_rate as f64)
	}

	/// Returns a clone of the `StaticSoundData` with the specified settings.
	pub fn with_settings(&self, settings: StaticSoundSettings) -> Self {
		Self {
			settings,
			..self.clone()
		}
	}

	/// Returns a clone of the `StaticSoundData` with the modified settings from
	/// the given function.
	pub fn with_modified_settings(
		&self,
		f: impl FnOnce(StaticSoundSettings) -> StaticSoundSettings,
	) -> Self {
		self.with_settings(f(self.settings))
	}

	pub(super) fn split(self) -> (StaticSound, StaticSoundHandle) {
		let (command_producer, command_consumer) = HeapRb::new(COMMAND_BUFFER_CAPACITY).split();
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

impl Debug for StaticSoundData {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("StaticSoundData")
			.field("sample_rate", &self.sample_rate)
			.field(
				"frames",
				&FramesDebug {
					len: self.frames.len(),
				},
			)
			.field("settings", &self.settings)
			.finish()
	}
}

struct FramesDebug {
	len: usize,
}

impl Debug for FramesDebug {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("[{} frames]", self.len))
	}
}
