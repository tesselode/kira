mod clip;

pub use clip::SoundClip;

use std::{
	hash::Hash,
	sync::atomic::{AtomicUsize, Ordering},
};

use indexmap::IndexMap;

use crate::{
	mixer::TrackIndex,
	playable::PlayableSettings,
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
	default_track: TrackIndex,
	semantic_duration: Option<f64>,
	default_loop_start: Option<f64>,
}

impl ArrangementId {
	pub(crate) fn new(arrangement: &Arrangement) -> Self {
		let index = NEXT_ARRANGEMENT_INDEX.fetch_add(1, Ordering::Relaxed);
		Self {
			index,
			duration: arrangement.duration(),
			default_track: arrangement.settings.default_track,
			semantic_duration: arrangement.settings.semantic_duration,
			default_loop_start: arrangement.settings.default_loop_start,
		}
	}

	pub fn duration(&self) -> f64 {
		self.duration
	}

	pub fn default_track(&self) -> TrackIndex {
		self.default_track
	}

	pub fn semantic_duration(&self) -> Option<f64> {
		self.semantic_duration
	}

	pub fn default_loop_start(&self) -> Option<f64> {
		self.default_loop_start
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
	pub settings: PlayableSettings,
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

	pub fn new_loop(sound_id: SoundId) -> Self {
		let duration = sound_id.semantic_duration().unwrap_or(sound_id.duration());
		Self::new(PlayableSettings::new().default_loop_start(duration))
			.add_clip(SoundClip::new(sound_id, 0.0))
			.add_clip(SoundClip::new(sound_id, duration).trim(duration))
	}

	pub fn add_clip(mut self, clip: SoundClip) -> Self {
		self.duration = self.duration.max(clip.clip_time_range.end);
		self.clips.push(clip);
		self
	}

	pub fn duration(&self) -> f64 {
		self.duration
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
