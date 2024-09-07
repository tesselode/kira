mod main;
mod send;
mod sub;

pub use main::*;
pub use send::*;
pub use sub::*;

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use crate::sound::PlaybackState;

#[derive(Debug)]
pub(crate) struct TrackShared {
	state: AtomicU8,
	removed: AtomicBool,
}

impl TrackShared {
	pub fn new() -> Self {
		Self {
			state: AtomicU8::new(TrackPlaybackState::Playing as u8),
			removed: AtomicBool::new(false),
		}
	}

	pub fn state(&self) -> TrackPlaybackState {
		match self.state.load(Ordering::SeqCst) {
			0 => TrackPlaybackState::Playing,
			1 => TrackPlaybackState::Pausing,
			2 => TrackPlaybackState::Paused,
			3 => TrackPlaybackState::WaitingToResume,
			4 => TrackPlaybackState::Resuming,
			_ => panic!("Invalid playback state"),
		}
	}

	pub fn set_state(&self, playback_state: PlaybackState) {
		self.state.store(playback_state as u8, Ordering::SeqCst);
	}

	#[must_use]
	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

/// The playback state of a mixer sub-track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TrackPlaybackState {
	/// The track is playing normally.
	Playing,
	/// The track is fading out, and when the fade-out
	/// is finished, playback will pause.
	Pausing,
	/// Playback is paused.
	Paused,
	/// The track is paused, but is schedule to resume in the future.
	WaitingToResume,
	/// The track is fading back in after being previously paused.
	Resuming,
}

impl TrackPlaybackState {
	/// Whether the track is outputting audio given
	/// its current playback state.
	pub fn is_advancing(self) -> bool {
		match self {
			TrackPlaybackState::Playing => true,
			TrackPlaybackState::Pausing => true,
			TrackPlaybackState::Paused => false,
			TrackPlaybackState::WaitingToResume => false,
			TrackPlaybackState::Resuming => true,
		}
	}
}

impl From<PlaybackState> for TrackPlaybackState {
	fn from(value: PlaybackState) -> Self {
		match value {
			PlaybackState::Playing => Self::Playing,
			PlaybackState::Pausing => Self::Pausing,
			PlaybackState::Paused => Self::Paused,
			PlaybackState::WaitingToResume => Self::WaitingToResume,
			PlaybackState::Resuming => Self::Resuming,
			PlaybackState::Stopping => unreachable!(),
			PlaybackState::Stopped => unreachable!(),
		}
	}
}
