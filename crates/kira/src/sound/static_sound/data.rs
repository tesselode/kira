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
	frame::Frame,
	sound::{Sound, SoundData},
	Panning,
};

use super::StaticSound;

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
	/**
	The portion of the sound this [`StaticSoundData`] represents.

	Note that the [`StaticSoundData`] holds the entire piece of audio
	it was originally given regardless of the value of `slice`, but
	[`StaticSoundData::num_frames`], [`StaticSoundData::duration`],
	and [`StaticSoundData::frame_at_index`] will all behave as if
	this [`StaticSoundData`] only contained the specified portion of
	audio.
	*/
	pub slice: Option<(usize, usize)>,
}

impl StaticSoundData {
	/// Returns the number of frames in the [`StaticSoundData`].
	///
	/// If [`StaticSoundData::slice`] is `Some`, this will be the number
	/// of frames in the slice.
	#[must_use]
	pub fn num_frames(&self) -> usize {
		num_frames(&self.frames, self.slice)
	}

	/// Returns the duration of the audio.
	///
	/// If [`StaticSoundData::slice`] is `Some`, this will be the duration
	/// of the slice.
	#[must_use]
	pub fn duration(&self) -> Duration {
		Duration::from_secs_f64(self.num_frames() as f64 / self.sample_rate as f64)
	}

	/// Returns the nth [`Frame`] of audio in the [`StaticSoundData`].
	///
	/// If [`StaticSoundData::slice`] is `Some`, this will behave as if the [`StaticSoundData`]
	/// only contained that portion of the audio.
	#[must_use]
	pub fn frame_at_index(&self, index: usize) -> Option<Frame> {
		frame_at_index(index, &self.frames, self.slice)
	}

	pub(super) fn split(self) -> (StaticSound, ()) {
		let sound = StaticSound::new(self);
		(sound, ())
	}
}

impl SoundData for StaticSoundData {
	type Error = ();

	type Handle = ();

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

pub(crate) fn num_frames(frames: &[Frame], slice: Option<(usize, usize)>) -> usize {
	if let Some((start, end)) = slice {
		end - start
	} else {
		frames.len()
	}
}

pub(crate) fn frame_at_index(
	index: usize,
	frames: &[Frame],
	slice: Option<(usize, usize)>,
) -> Option<Frame> {
	if index >= num_frames(frames, slice) {
		return None;
	}
	let start = slice.map(|(start, _)| start).unwrap_or_default();
	Some(frames[index + start])
}
