//! Organizes and applies effects to audio.
//!
//! All sounds in Kira are processed by the mixer before
//! being sent to the operating system's audio driver.
//! The mixer is made up of a number of **tracks**, which are
//! like continuously flowing rivers of audio data. Each
//! track can have a series of **effects**, which are
//! processors that transform input audio in some way
//! and output the transformed audio.
//!
//! There are three kinds of mixer tracks:
//! - The **main track**, which all audio eventually reaches
//! - **Sub-tracks**, which can each have their own volume
//! levels, sets of effects, and parent tracks (either the main
//! track or another sub-track)
//! - **Send tracks**, which sub-tracks can be additionally
//! routed to. These are useful for applying effects to sounds
//! from multiple sub-tracks.
//!
//! Each instance of a sound or arrangement can have its audio
//! sent to the main track, a sub-track, or a send track.
//!
//! Here's an example of a way you might want to set up a mixer:
//!
//! ```text
//! ┌──Sub──┐    ┌──Sub──┐    ┌──Sub───┐
//! │ Music │    │  SFX  │    │Ambience├─────┐
//! └───┬───┘    └───┬─┬─┘    └────┬───┘     │
//!     │            │ │           │         ▼
//!     │            │ │           │      ┌─Send─┐
//!     │            │ └───────────┼─────►│Reverb│
//!     │            │             │      └──┬───┘
//!     │     ┌──────▼─────┐       │         │
//!     │     │            ◄───────┘         │
//!     └─────► Main track │                 │
//!           │            ◄─────────────────┘
//!           └────────────┘
//! ```
//!
//! Using sub-tracks for different categories of audio allows for
//! easy control of the volume levels of the music, sfx, etc.
//! Using a send track for reverb allows you to use the same
//! reverb effect for multiple kinds of audio, reducing CPU
//! usage and code repetition (of course, this only applies if you
//! want SFX and ambience to have the same reverb effect).
//!
//! For each track, audio is fed through each effect in order.
//! The signal flow for a track with a filter added first and then
//! a reverb looks like this:
//!
//! ```text
//!   Input audio
//!        │
//!        ▼
//! ┌────Track─────┐
//! │      │       │
//! │   ┌──▼───┐   │
//! │   │Filter│   │
//! │   └──┬───┘   │
//! │      │       │
//! │   ┌──▼───┐   │
//! │   │Reverb│   │
//! │   └──┬───┘   │
//! │      │       │
//! └──────┼───────┘
//!        │
//!        ▼
//!   Output audio
//! ```

pub mod effect;
pub(crate) mod effect_slot;
mod track;

pub use track::{
	handle::{MainTrackHandle, SendTrackHandle, SubTrackHandle},
	sends::TrackSends,
	SendTrackId, SendTrackSettings, SubTrackId, SubTrackSettings, TrackIndex,
};
pub(crate) use track::{Track, TrackKind};
