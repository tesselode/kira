/*!
Sources of audio.

Any type that implements [`SoundData`] can be played using
[`AudioManager::play`](crate::manager::AudioManager::play). Kira comes with two
[`SoundData`] implementations:

- [`StaticSoundData`](static_sound::StaticSoundData), which loads an entire chunk of audio
into memory
- [`StreamingSoundData`](streaming::StreamingSoundData), which streams audio from a file or cursor
(only available on desktop platforms)

These two sound types should cover most use cases, but if you need something else, you can
create your own types that implement the [`SoundData`] and [`Sound`] traits.
*/

mod common;
#[cfg(feature = "symphonia")]
mod error;
mod playback_position;
mod playback_rate;
/* pub mod static_sound;
#[cfg(not(target_arch = "wasm32"))]
pub mod streaming; */
#[cfg(feature = "symphonia")]
mod symphonia;
mod transport;
pub(crate) mod wrapper;

use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

pub use common::*;
#[cfg(feature = "symphonia")]
pub use error::*;
pub use playback_position::*;
pub use playback_rate::*;

use crate::{
	clock::clock_info::ClockInfoProvider, dsp::Frame,
	modulator::value_provider::ModulatorValueProvider,
};

/// A source of audio that is loaded, but not yet playing.
pub trait SoundData {
	/// Errors that can occur when starting the sound.
	type Error;

	/// The type that can be used to control the sound once
	/// it has started.
	type Handle;

	/// Returns the common settings for this sound.
	fn common_settings(&self) -> CommonSoundSettings;

	/// Converts the loaded sound into a live, playing sound
	/// and a handle to control it.
	///
	/// The [`Sound`] implementation will be sent to the audio renderer
	/// for playback, and the handle will be returned to the user by
	/// [`AudioManager::play`](crate::manager::AudioManager::play).
	#[allow(clippy::type_complexity)]
	fn into_sound(
		self,
		common_controller: CommonSoundController,
	) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error>;
}

/// An actively playing sound.
///
/// For performance reasons, the methods of this trait should not allocate
/// or deallocate memory.
#[allow(unused_variables)]
pub trait Sound: Send {
	/// Returns the sample rate of the sound (in Hz).
	///
	/// This can change over time to affect a sound's playback rate.
	fn sample_rate(&self) -> f64;

	/// Called whenever a new batch of audio samples is requested by the backend.
	///
	/// This is a good place to put code that needs to run fairly frequently,
	/// but not for every single audio sample.
	fn on_start_processing(&mut self) {}

	/// Produces the next [`Frame`] of audio.
	fn process(
		&mut self,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame;

	/// Returns `true` if the sound is finished and can be unloaded.
	///
	/// For finite sounds, this will typically be when playback has reached the
	/// end of the sound. For infinite sounds, this will typically be when the
	/// handle for the sound is dropped.
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
	/// The sound is fading out, and when the fade-out
	/// is finished, playback will stop.
	Stopping,
	/// The sound has stopped and can no longer be resumed.
	Stopped,
}

/// A portion of audio.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Region {
	/// The starting time of the region (in seconds).
	pub start: PlaybackPosition,
	/// The ending time of the region.
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

impl<T: Into<PlaybackPosition> + Copy> From<RangeInclusive<T>> for Region {
	fn from(range: RangeInclusive<T>) -> Self {
		Self {
			start: (*range.start()).into(),
			end: EndPosition::Custom((*range.end()).into()),
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

impl<T: Into<PlaybackPosition>> From<RangeToInclusive<T>> for Region {
	fn from(range: RangeToInclusive<T>) -> Self {
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

impl Default for Region {
	fn default() -> Self {
		Self {
			start: PlaybackPosition::Samples(0),
			end: EndPosition::EndOfAudio,
		}
	}
}

/// A trait for types that can be converted into an `Option<Region>`.
pub trait IntoOptionalRegion {
	/// Converts the type into an `Option<Region>`.
	fn into_optional_loop_region(self) -> Option<Region>;
}

impl<T: Into<Region>> IntoOptionalRegion for T {
	fn into_optional_loop_region(self) -> Option<Region> {
		Some(self.into())
	}
}

impl IntoOptionalRegion for Option<Region> {
	fn into_optional_loop_region(self) -> Option<Region> {
		self
	}
}

/// The ending time of a region of audio.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EndPosition {
	/// The end of the audio data.
	EndOfAudio,
	/// A user-defined time in seconds.
	Custom(PlaybackPosition),
}
