//! Decodes data gradually from an audio file.

#![cfg_attr(docsrs, doc(cfg(not(wasm32))))]

mod data;
mod decoder;
mod handle;
mod settings;
mod sound;

pub use data::*;
pub use decoder::*;
pub use handle::*;
pub use settings::*;

use crate::{parameter::Value, tween::Tween, PlaybackRate, Volume};

use super::{LoopRegion, PlaybackRegion};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SoundCommand {
	SetVolume(Value<Volume>, Tween),
	SetPlaybackRate(Value<PlaybackRate>, Tween),
	SetPanning(Value<f64>, Tween),
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DecodeSchedulerCommand {
	SetPlaybackRegion(PlaybackRegion),
	SetLoopRegion(Option<LoopRegion>),
	SeekBy(f64),
	SeekTo(f64),
}
