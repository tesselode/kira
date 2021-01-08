//! Provides an interface for sending arbitrary audio
//! data to a mixer track.
//!
//! Audio streams are useful if you need to dynamically generate
//! audio. For instance, you can use an audio stream to synthesize
//! sound effects in real time or feed audio from a voice chat
//! into the mixer.
//!
//! If you just need to play an audio file, you should probably use
//! [instances](crate::instance).

use std::fmt::Debug;

use uuid::Uuid;

use crate::{util::generate_uuid, Frame};

/// Produces a constant flow of audio data in real time.
pub trait AudioStream: Debug + Send + 'static {
	/// Produces the next sample.
	///
	/// The audio thread has to wait for this function to finish,
	/// so it should process quickly and in a consistent amount
	/// of time to avoid audio glitches, such as stuttering.
	///
	/// `dt` represents how many seconds have elapsed since the last request.
	fn next(&mut self, dt: f64) -> Frame;
}

/// A unique identifier for an [`AudioStream`](crate::audio_stream::AudioStream).
///
/// You cannot create this manually - an audio stream ID is returned
/// when you start an audio stream with an [`AudioManager`](crate::manager::AudioManager).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
pub struct AudioStreamId {
	uuid: Uuid,
}

impl AudioStreamId {
	pub(crate) fn new() -> Self {
		Self {
			uuid: generate_uuid(),
		}
	}
}
