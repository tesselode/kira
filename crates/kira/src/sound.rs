pub mod sine;

use crate::{info::Info, Frame};

/// A source of audio that is loaded, but not yet playing.
pub trait SoundData {
	/// Errors that can occur when starting the sound.
	type Error;

	/// The type that can be used to control the sound once
	/// it has started.
	type Handle;

	/// Converts the loaded sound into a live, playing sound
	/// and a handle to control it.
	///
	/// The [`Sound`] implementation will be sent to the audio renderer
	/// for playback, and the handle will be returned to the user by
	/// [`AudioManager::play`](crate::manager::AudioManager::play).
	#[allow(clippy::type_complexity)]
	fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error>;
}

/// An actively playing sound.
///
/// For performance reasons, the methods of this trait should not allocate
/// or deallocate memory.
#[allow(unused_variables)]
pub trait Sound: Send {
	/// Called whenever a new batch of audio samples is requested by the backend.
	///
	/// This is a good place to put code that needs to run fairly frequently,
	/// but not for every single audio sample.
	fn on_start_processing(&mut self) {}

	/// Produces the next [`Frame`] of audio.
	///
	/// `dt` is the time that's elapsed since the previous round of
	/// processing (in seconds).
	fn process(&mut self, out: &mut [Frame], dt: f64, info: &Info);

	/// Returns `true` if the sound is finished and can be unloaded.
	///
	/// For finite sounds, this will typically be when playback has reached the
	/// end of the sound. For infinite sounds, this will typically be when the
	/// handle for the sound is dropped.
	#[must_use]
	fn finished(&self) -> bool;
}

/// The playback state of a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlaybackState {
	/// The sound is playing normally.
	Playing,
	/// The sound is fading out, and when the fade-out
	/// is finished, playback will pause.
	Pausing,
	/// Playback is paused.
	Paused,
	/// The sound is paused, but is schedule to resume in the future.
	WaitingToResume,
	/// The sound is fading back in after being previously paused.
	Resuming,
	/// The sound is fading out, and when the fade-out
	/// is finished, playback will stop.
	Stopping,
	/// The sound has stopped and can no longer be resumed.
	Stopped,
}

impl PlaybackState {
	/// Whether the sound is advancing and outputting audio given
	/// its current playback state.
	pub fn is_advancing(self) -> bool {
		match self {
			PlaybackState::Playing => true,
			PlaybackState::Pausing => true,
			PlaybackState::Paused => false,
			PlaybackState::WaitingToResume => false,
			PlaybackState::Resuming => true,
			PlaybackState::Stopping => true,
			PlaybackState::Stopped => false,
		}
	}
}
