pub mod svf;

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{manager::backend::parameters::Parameters, stereo_sample::StereoSample};

use super::index::TrackIndex;

static NEXT_EFFECT_INDEX: AtomicUsize = AtomicUsize::new(0);

/**
A unique identifier for an `Effect`.

You cannot create this manually - an `EffectId` is created
when you add an effect to a mixer track with an `AudioManager`.
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

pub trait Effect: Send {
	fn process(&mut self, dt: f64, input: StereoSample, parameters: &Parameters) -> StereoSample;
}
