/*!
Sources of audio.

If you just want to load sounds from audio files (mp3, ogg, etc.),
take a look at the documentation for
[`kira-symphonia`](https://crates.io/crates/kira-symphonia).

## Implementing [`Sound`] and [`SoundData`]

Sounds in Kira have two phases:
1. The [`SoundData`] phase: the user has created a sound, but it is
not yet producing sound on the audio thread. If the sound data has
settings, they should still be customizable at this point.
2. The [`Sound`] phase: the user has played the sound using
[`AudioManager::play`](crate::manager::AudioManager::play), which
transfers ownership to the audio thread.

The [`SoundData`] trait has the [`into_sound`](SoundData::into_sound)
function, which "splits" the sound data into the live [`Sound`]
and a [`Handle`](SoundData::Handle) which the user can use to control
the sound from gameplay code.

[`Sound`]s simply produce a [`Frame`] of audio each time
[`process`](Sound::process) is called. A [`Sound`] can be a finite
chunk of audio, an infinite stream of audio (e.g. voice chat),
or anything else.

Kira does not provide any tools for passing messages from gameplay
code to a [`Sound`] or vice versa. (Internally, Kira uses the
[`ringbuf`](https://crates.io/crates/ringbuf) crate for this purpose.)
*/

pub mod static_sound;

use crate::{clock::Clocks, dsp::Frame, parameter::Parameters, track::TrackId};

/// Represents a source of audio that is loaded, but not yet playing.
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

/// Represents an actively playing sound.
pub trait Sound: Send {
	/// Returns the mixer track that this sound's audio should be routed to.
	fn track(&mut self) -> TrackId;

	/// Called whenever a new batch of audio samples is requested by the backend.
	///
	/// This is a good place to put code that needs to run fairly frequently,
	/// but not for every single audio sample.
	fn on_start_processing(&mut self) {}

	/// Produces the next [`Frame`] of audio.
	fn process(&mut self, dt: f64, parameters: &Parameters, clocks: &Clocks) -> Frame;

	/// Returns `true` if the sound is finished and can be unloaded.
	fn finished(&self) -> bool;
}
