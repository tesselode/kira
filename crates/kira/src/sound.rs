/*!
Sources of audio.

Any type that implements [`SoundData`] can be played using
[`AudioManager::play`](crate::manager::AudioManager::play). Kira comes with two
[`SoundData`] implementations:

- [`StaticSoundData`](static_sound::StaticSoundData), which loads an entire chunk of audio
into memory. This is more appropriate for short sounds, sounds you want to play multiple times,
or sounds where consistent start times are important.
- [`StreamingSoundData`](streaming::StreamingSoundData), which streams audio from a file or cursor
(only available on desktop platforms). This is more appropriate for long sounds that you only
play once at a time, like background music. Streaming sounds use less memory than static sounds.

These two sound types should cover most use cases, but if you need something else, you can
create your own types that implement the [`SoundData`] and [`Sound`] traits.
*/

#[cfg(feature = "symphonia")]
mod error;
mod playback_position;
mod playback_rate;
pub mod static_sound;
#[cfg(not(target_arch = "wasm32"))]
pub mod streaming;
#[cfg(feature = "symphonia")]
mod symphonia;
mod transport;

use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

#[cfg(feature = "symphonia")]
pub use error::*;
pub use playback_position::*;
pub use playback_rate::*;

use crate::{frame::Frame, info::Info};

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
	fn process(&mut self, dt: f64, info: &Info) -> Frame;

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

/// A portion of audio.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Region {
	/// The starting time of the region (in seconds).
	pub start: PlaybackPosition,
	/// The (exclusive) ending time of the region.
	pub end: EndPosition,
}

impl<T: Into<PlaybackPosition>> From<RangeFrom<T>> for Region {
	fn from(range: RangeFrom<T>) -> Self {
		Self {
			start: range.start.into(),
			end: EndPosition::EndOfAudio,
		}
	}
}

impl<T: Into<PlaybackPosition>> From<Range<T>> for Region {
	fn from(range: Range<T>) -> Self {
		Self {
			start: range.start.into(),
			end: EndPosition::Custom(range.end.into()),
		}
	}
}

impl<T: Into<PlaybackPosition>> From<RangeTo<T>> for Region {
	fn from(range: RangeTo<T>) -> Self {
		Self {
			start: PlaybackPosition::Samples(0),
			end: EndPosition::Custom(range.end.into()),
		}
	}
}

impl From<RangeFull> for Region {
	fn from(_: RangeFull) -> Self {
		Self {
			start: PlaybackPosition::Samples(0),
			end: EndPosition::EndOfAudio,
		}
	}
}

/// A trait for types that can be converted into an `Option<Region>`.
pub trait IntoOptionalRegion {
	/// Converts the type into an `Option<Region>`.
	#[must_use]
	fn into_optional_region(self) -> Option<Region>;
}

impl<T: Into<Region>> IntoOptionalRegion for T {
	fn into_optional_region(self) -> Option<Region> {
		Some(self.into())
	}
}

impl IntoOptionalRegion for Option<Region> {
	fn into_optional_region(self) -> Option<Region> {
		self
	}
}

/// The ending time of a region of audio.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EndPosition {
	/// The end of the audio data.
	#[default]
	EndOfAudio,
	/// A user-defined time in seconds.
	Custom(PlaybackPosition),
}
