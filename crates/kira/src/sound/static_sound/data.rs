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
	sound::{EndPosition, IntoOptionalRegion, Region, Sound, SoundData},
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
	pub slice: Option<(usize, usize)>,
}

impl StaticSoundData {
	pub fn num_frames(&self) -> usize {
		if let Some((start, end)) = self.slice {
			end - start
		} else {
			self.frames.len()
		}
	}

	/// Returns the duration of the audio.
	pub fn duration(&self) -> Duration {
		Duration::from_secs_f64(self.num_frames() as f64 / self.sample_rate as f64)
	}

	pub fn frame_at_index(&self, index: usize) -> Option<Frame> {
		if index >= self.num_frames() {
			return None;
		}
		let start = self.slice.map(|(start, _)| start).unwrap_or_default();
		Some(self.frames[index + start])
	}

	pub fn slice(&self, region: impl IntoOptionalRegion) -> Self {
		let mut new = self.clone();
		new.slice = region.into_optional_region().map(|Region { start, end }| {
			let start = start.into_samples(self.sample_rate) as usize;
			let end = match end {
				EndPosition::EndOfAudio => self.frames.len(),
				EndPosition::Custom(end) => end.into_samples(self.sample_rate) as usize,
			};
			(start, end)
		});
		new
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
