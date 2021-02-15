use basedrop::Owned;
use indexmap::IndexMap;

use crate::{
	sound::{handle::SoundHandle, Sound, SoundId},
	util::inverse_lerp,
	util::lerp,
	Frame,
};

/// A segment of a sound in an arrangement.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize)
)]
pub struct SoundClip {
	/// The ID of the sound.
	pub sound_id: SoundId,
	/// The start and end point of the clip.
	pub clip_time_range: (f64, f64),
	/// The start and end point of the sound.
	///
	/// This range of the sound is stretched over
	/// the range of the clip.
	pub sound_time_range: (f64, f64),
}

impl SoundClip {
	/// Creates a new sound clip that starts at the specified time
	/// and contains the whole sound without any cropping or
	/// speed up/slow down.
	pub fn new(sound_handle: &SoundHandle, clip_start_time: f64) -> Self {
		Self {
			sound_id: sound_handle.id(),
			clip_time_range: (clip_start_time, clip_start_time + sound_handle.duration()),
			sound_time_range: (0.0, sound_handle.duration()),
		}
	}

	/// Gets the duration of the sound clip.
	pub fn duration(&self) -> f64 {
		self.clip_time_range.1 - self.clip_time_range.0
	}

	/// Increases the length of the clip by the given factor.
	///
	/// A factor greater than 1 will slow down the sound,
	/// and a factor less than 1 will speed it up.
	pub fn stretch(mut self, factor: f64) -> Self {
		self.clip_time_range.1 = lerp(self.clip_time_range.0, self.clip_time_range.1, factor);
		self
	}

	/// Sets the duration of the clip, preserving the pitch
	/// of the sound.
	pub fn trim(mut self, duration: f64) -> Self {
		let new_duration_factor = duration / self.duration();
		self.clip_time_range.1 = self.clip_time_range.0 + duration;
		self.sound_time_range.1 = lerp(
			self.sound_time_range.0,
			self.sound_time_range.1,
			new_duration_factor,
		);
		self
	}

	/// Gets the frame that this clip will output at a given time.
	///
	/// If the time is outside of the clip's time range, no sound
	/// will be produced.
	pub(crate) fn get_frame_at_position(
		&self,
		position: f64,
		sounds: &IndexMap<SoundId, Owned<Sound>>,
	) -> Frame {
		if let Some(sound) = sounds.get(&self.sound_id) {
			let relative_time =
				inverse_lerp(self.clip_time_range.0, self.clip_time_range.1, position);
			if relative_time < 0.0 || relative_time > 1.0 {
				Frame::from_mono(0.0)
			} else {
				sound.get_frame_at_position(lerp(
					self.sound_time_range.0,
					self.sound_time_range.1,
					relative_time,
				))
			}
		} else {
			Frame::from_mono(0.0)
		}
	}
}
