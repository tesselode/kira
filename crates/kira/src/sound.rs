/*!
Sources of audio.
*/

#[cfg(feature = "symphonia")]
mod error;
pub mod static_sound;
#[cfg(not(target_arch = "wasm32"))]
pub mod streaming;
#[cfg(feature = "symphonia")]
mod symphonia;
mod transport;
mod util;

use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive};

#[cfg(feature = "symphonia")]
pub use error::*;

use crate::{
	clock::clock_info::ClockInfoProvider, dsp::Frame,
	modulator::value_provider::ModulatorValueProvider, OutputDestination,
};

/// A source of audio that is loaded, but not yet playing.
pub trait SoundData {
	/// Errors that can occur when starting the sound.
	type Error;

	/// The type that can be used to control the sound once
	/// it has started.
	type Handle;

	/// Converts the loaded sound into a live, playing sound
	/// and a handle to control it.
	#[allow(clippy::type_complexity)]
	fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error>;
}

/// An actively playing sound.
#[allow(unused_variables)]
pub trait Sound: Send {
	/// Returns the destination that this sound's audio should be routed to.
	fn output_destination(&mut self) -> OutputDestination;

	/// Called whenever a new batch of audio samples is requested by the backend.
	///
	/// This is a good place to put code that needs to run fairly frequently,
	/// but not for every single audio sample.
	fn on_start_processing(&mut self) {}

	/// Produces the next [`Frame`] of audio.
	fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame;

	/// Returns `true` if the sound is finished and can be unloaded.
	fn finished(&self) -> bool;
}

/// The playback state of a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Region {
	pub start: f64,
	pub end: EndPosition,
}

impl From<RangeFrom<f64>> for Region {
	fn from(range: RangeFrom<f64>) -> Self {
		Self {
			start: range.start,
			end: EndPosition::EndOfAudio,
		}
	}
}

impl From<Range<f64>> for Region {
	fn from(range: Range<f64>) -> Self {
		Self {
			start: range.start,
			end: EndPosition::Custom(range.end),
		}
	}
}

impl From<RangeInclusive<f64>> for Region {
	fn from(range: RangeInclusive<f64>) -> Self {
		Self {
			start: *range.start(),
			end: EndPosition::Custom(*range.end()),
		}
	}
}

impl From<RangeFull> for Region {
	fn from(_: RangeFull) -> Self {
		Self {
			start: 0.0,
			end: EndPosition::EndOfAudio,
		}
	}
}

impl Default for Region {
	fn default() -> Self {
		Self {
			start: 0.0,
			end: EndPosition::EndOfAudio,
		}
	}
}

pub trait IntoOptionalRegion {
	fn into_optional_loop_region(self) -> Option<Region>;
}

impl IntoOptionalRegion for RangeFrom<f64> {
	fn into_optional_loop_region(self) -> Option<Region> {
		Some(Region {
			start: self.start,
			end: EndPosition::EndOfAudio,
		})
	}
}

impl IntoOptionalRegion for Range<f64> {
	fn into_optional_loop_region(self) -> Option<Region> {
		Some(Region {
			start: self.start,
			end: EndPosition::Custom(self.end),
		})
	}
}

impl IntoOptionalRegion for RangeInclusive<f64> {
	fn into_optional_loop_region(self) -> Option<Region> {
		Some(Region {
			start: *self.start(),
			end: EndPosition::Custom(*self.end()),
		})
	}
}

impl IntoOptionalRegion for RangeFull {
	fn into_optional_loop_region(self) -> Option<Region> {
		Some(Region {
			start: 0.0,
			end: EndPosition::EndOfAudio,
		})
	}
}

impl IntoOptionalRegion for Option<Region> {
	fn into_optional_loop_region(self) -> Option<Region> {
		self
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EndPosition {
	EndOfAudio,
	Custom(f64),
}
