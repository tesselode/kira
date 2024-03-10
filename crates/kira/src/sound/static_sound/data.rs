#[cfg(feature = "symphonia")]
mod from_file;

#[cfg(test)]
mod test;

use std::{
	fmt::{Debug, Formatter},
	sync::Arc,
	time::Duration,
};

use crate::{
	dsp::Frame,
	sound::{CommonSoundController, CommonSoundSettings, EndPosition, Region, Sound, SoundData},
};

use super::{
	command_writers_and_readers, handle::StaticSoundHandle, sound::StaticSound, StaticSoundSettings,
};

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

	pub fn sliced(&self, region: impl Into<Region>) -> Self {
		let Region { start, end } = region.into();
		let slice = (
			start.into_samples(self.sample_rate),
			match end {
				EndPosition::EndOfAudio => self.frames.len(),
				EndPosition::Custom(end) => end.into_samples(self.sample_rate),
			},
		);
		Self {
			slice: Some(slice),
			..self.clone()
		}
	}

	pub fn frame(&self, index: usize) -> Frame {
		frame(&self.frames, self.slice, index)
	}

	pub fn num_frames(&self) -> usize {
		num_frames(&self.frames, self.slice)
	}

	/// Returns the duration of the audio.
	pub fn duration(&self) -> Duration {
		Duration::from_secs_f64(self.num_frames() as f64 / self.sample_rate as f64)
	}

	pub(super) fn split(
		self,
		common_controller: CommonSoundController,
	) -> (StaticSound, StaticSoundHandle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		let sound = StaticSound::new(self, command_readers);
		let shared = sound.shared();
		(
			sound,
			StaticSoundHandle {
				common_controller,
				shared,
				command_writers,
			},
		)
	}
}

impl SoundData for StaticSoundData {
	type Error = ();

	type Handle = StaticSoundHandle;

	fn common_settings(&self) -> CommonSoundSettings {
		CommonSoundSettings {
			start_time: self.settings.start_time,
			volume: self.settings.volume,
			panning: self.settings.panning,
			output_destination: self.settings.output_destination,
			fade_in_tween: self.settings.fade_in_tween,
		}
	}

	#[allow(clippy::type_complexity)]
	fn into_sound(
		self,
		common_controller: CommonSoundController,
	) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
		let (sound, handle) = self.split(common_controller);
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

pub(super) fn frame(frames: &[Frame], slice: Option<(usize, usize)>, index: usize) -> Frame {
	let start = slice.map(|(start, _)| start).unwrap_or(0);
	let end = slice.map(|(_, end)| end).unwrap_or(frames.len());
	if index >= end - start {
		return Frame::ZERO;
	}
	let absolute_index = start + index;
	if absolute_index >= frames.len() {
		return Frame::ZERO;
	}
	frames[absolute_index]
}

pub(super) fn num_frames(frames: &[Frame], slice: Option<(usize, usize)>) -> usize {
	slice
		.map(|(start, end)| end - start)
		.unwrap_or(frames.len())
}
