mod clip;

pub use clip::SoundClip;

use std::{
	hash::Hash,
	sync::atomic::{AtomicUsize, Ordering},
};

use indexmap::IndexMap;

use crate::{
	mixer::TrackIndex,
	playable::{Metadata, PlayableSettings},
	sound::{Sound, SoundId},
	Frame,
};

static NEXT_ARRANGEMENT_INDEX: AtomicUsize = AtomicUsize::new(0);

/**
A unique identifier for an [arrangement](Arrangement).

You cannot create this manually - an arrangement ID is created
when you create a arrangement with an [`AudioManager`](crate::manager::AudioManager).
*/
#[derive(Debug, Copy, Clone)]
pub struct ArrangementId {
	index: usize,
	duration: f64,
	default_track_index: TrackIndex,
	metadata: Metadata,
}

impl ArrangementId {
	pub(crate) fn new(duration: f64, default_track_index: TrackIndex, metadata: Metadata) -> Self {
		let index = NEXT_ARRANGEMENT_INDEX.fetch_add(1, Ordering::Relaxed);
		Self {
			index,
			duration,
			default_track_index,
			metadata,
		}
	}

	pub fn duration(&self) -> f64 {
		self.duration
	}

	pub fn default_track_index(&self) -> TrackIndex {
		self.default_track_index
	}

	pub fn metadata(&self) -> &Metadata {
		&self.metadata
	}
}

impl PartialEq for ArrangementId {
	fn eq(&self, other: &Self) -> bool {
		self.index == other.index
	}
}

impl Eq for ArrangementId {}

impl Hash for ArrangementId {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.index.hash(state);
	}
}

#[derive(Debug, Clone)]
pub struct Arrangement {
	clips: Vec<SoundClip>,
	duration: f64,
	settings: PlayableSettings,
	cooldown_timer: f64,
}

impl Arrangement {
	pub fn new(settings: PlayableSettings) -> Self {
		Self {
			clips: vec![],
			duration: 0.0,
			settings,
			cooldown_timer: 0.0,
		}
	}

	pub fn add_clip(mut self, clip: SoundClip) -> Self {
		self.duration = self.duration.max(clip.clip_time_range.end);
		self.clips.push(clip);
		self
	}

	pub fn duration(&self) -> f64 {
		self.duration
	}

	/// Gets the default track that the sound plays on.
	pub fn default_track(&self) -> TrackIndex {
		self.settings.default_track
	}

	pub(crate) fn get_frame_at_position(
		&self,
		position: f64,
		sounds: &IndexMap<SoundId, Sound>,
	) -> Frame {
		let mut frame = Frame::from_mono(0.0);
		for clip in &self.clips {
			frame += clip.get_frame_at_position(position, sounds);
		}
		frame
	}

	/// Gets the metadata associated with the sound.
	pub fn metadata(&self) -> Metadata {
		self.settings.metadata
	}

	pub(crate) fn start_cooldown(&mut self) {
		if let Some(cooldown) = self.settings.cooldown {
			self.cooldown_timer = cooldown;
		}
	}

	pub(crate) fn update_cooldown(&mut self, dt: f64) {
		if self.cooldown_timer > 0.0 {
			self.cooldown_timer -= dt;
		}
	}

	pub(crate) fn cooling_down(&self) -> bool {
		self.cooldown_timer > 0.0
	}
}
