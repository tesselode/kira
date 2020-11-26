pub mod filter;

use std::{
	fmt::Debug,
	sync::atomic::{AtomicUsize, Ordering},
};

use crate::{frame::Frame, parameter::Parameters};

use super::TrackIndex;

static NEXT_EFFECT_INDEX: AtomicUsize = AtomicUsize::new(0);

/**
A unique identifier for an effect.

You cannot create this manually - an effect ID is created
when you add an effect to a mixer track with an [`AudioManager`](crate::manager::AudioManager).
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct EffectId {
	index: usize,
	track_index: TrackIndex,
}

impl EffectId {
	pub(crate) fn new(track_index: TrackIndex) -> Self {
		let index = NEXT_EFFECT_INDEX.fetch_add(1, Ordering::Relaxed);
		Self { index, track_index }
	}

	/// Gets the mixer track that this effect applies to.
	pub fn track_index(&self) -> TrackIndex {
		self.track_index
	}
}

/// Settings for an effect.
#[derive(Debug, Clone)]
pub struct EffectSettings {
	/// Whether the effect is initially enabled.
	pub enabled: bool,
}

impl Default for EffectSettings {
	fn default() -> Self {
		Self { enabled: true }
	}
}

pub trait Effect: Send + Debug {
	fn process(&mut self, dt: f64, input: Frame, parameters: &Parameters) -> Frame;
}
