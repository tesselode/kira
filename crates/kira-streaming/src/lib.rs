//! # kira-streaming
//!
//! kira-streaming provides a common interface for streaming audio
//! from disk.

#![warn(missing_docs)]

mod data;
mod handle;
mod settings;
mod sound;

pub use data::*;
pub use handle::*;
pub use settings::*;

use std::collections::VecDeque;

use kira::{dsp::Frame, tween::Tween, value::Value};

/// Produces packets of audio on request.
pub trait Decoder: Send + Sync {
	/// Errors that the decoder can return.
	type Error: Send + Sync;

	/// Returns the sample rate of the audio.
	fn sample_rate(&mut self) -> u32;

	/// Returns the next handful of [`Frame`]s of audio to be played,
	/// or `None` if the end of the audio has been reached.
	fn decode(&mut self) -> Result<Option<VecDeque<Frame>>, Self::Error>;

	/// Resets the decoder to the beginning of the audio.
	fn reset(&mut self) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Command {
	SetVolume(Value),
	SetPlaybackRate(Value),
	SetPanning(Value),
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
	SeekBy(f64),
	SeekTo(f64),
}
