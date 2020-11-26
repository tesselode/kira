//! Provides an interface for "stitching together" individual sounds
//! into larger pieces.

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
A unique identifier for an [`Arrangement`](Arrangement).

You cannot create this manually - an arrangement ID is created
when you [add an arrangement](crate::manager::AudioManager::add_arrangement)
to an [`AudioManager`](crate::manager::AudioManager).
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

	/// Gets the duration of the arrangement.
	pub fn duration(&self) -> f64 {
		self.duration
	}

	/// Gets the default track that instances of this arrangement
	/// will play on.
	pub fn default_track(&self) -> TrackIndex {
		self.default_track
	}

	/// Gets the [semantic duration](crate::playable::PlayableSettings#structfield.semantic_duration)
	/// of the arrangement.
	pub fn semantic_duration(&self) -> Option<f64> {
		self.semantic_duration
	}

	/// Gets the default loop start point for instances of this
	/// arrangement.
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

/// An arrangement of sound clips to play at specific times.
#[derive(Debug, Clone)]
pub struct Arrangement {
	clips: Vec<SoundClip>,
	duration: f64,
	/// Settings for the arrangement.
	pub settings: PlayableSettings,
	cooldown_timer: f64,
}

impl Arrangement {
	/// Creates a new, empty arrangement.
	pub fn new(settings: PlayableSettings) -> Self {
		Self {
			clips: vec![],
			duration: 0.0,
			settings,
			cooldown_timer: 0.0,
		}
	}

	/// Creates a new arrangement that seamlessly loops a sound.
	///
	/// If the sound has a semantic duration, it will be used to
	/// set the point where the sound loops. Any audio after the loop
	/// point will be preserved when the loop starts.
	pub fn new_loop(sound_id: SoundId) -> Self {
		let duration = sound_id.semantic_duration().unwrap_or(sound_id.duration());
		Self::new(PlayableSettings::new().default_loop_start(duration))
			.add_clip(SoundClip::new(sound_id, 0.0))
			.add_clip(SoundClip::new(sound_id, duration).trim(duration))
	}

	/// Creates a new arrangement that plays an intro sound, then
	/// seamlessly loops another sound.
	///
	/// If the intro has a semantic duration, it will be used to determine
	/// when the loop sound starts. If the loop sound has a semantic duration,
	/// it will be used to set the point where the sound repeats. Any audio
	/// after the loop point will be preserved when the sound repeats.
	pub fn new_loop_with_intro(intro_sound_id: SoundId, loop_sound_id: SoundId) -> Self {
		let intro_duration = intro_sound_id
			.semantic_duration()
			.unwrap_or(intro_sound_id.duration());
		let loop_duration = loop_sound_id
			.semantic_duration()
			.unwrap_or(loop_sound_id.duration());
		Self::new(PlayableSettings::new().default_loop_start(intro_duration + loop_duration))
			.add_clip(SoundClip::new(intro_sound_id, 0.0))
			.add_clip(SoundClip::new(loop_sound_id, intro_duration))
			.add_clip(
				SoundClip::new(loop_sound_id, intro_duration + loop_duration).trim(loop_duration),
			)
	}

	/// Adds a sound clip to the arrangement.
	pub fn add_clip(mut self, clip: SoundClip) -> Self {
		self.duration = self.duration.max(clip.clip_time_range.1);
		self.clips.push(clip);
		self
	}

	/// Gets the duration of the arrangement.
	///
	/// The duration is always the end of the last playing sound clip.
	pub fn duration(&self) -> f64 {
		self.duration
	}

	/// Gets the frame at the given position of the arrangement.
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

	/// Starts the cooldown timer for the arrangement.
	pub(crate) fn start_cooldown(&mut self) {
		if let Some(cooldown) = self.settings.cooldown {
			self.cooldown_timer = cooldown;
		}
	}

	/// Updates the cooldown timer for the arrangement.
	pub(crate) fn update_cooldown(&mut self, dt: f64) {
		if self.cooldown_timer > 0.0 {
			self.cooldown_timer -= dt;
		}
	}

	/// Gets whether the arrangement is currently "cooling down".
	///
	/// If it is, a new instance of the arrangement should not
	/// be started until the timer is up.
	pub(crate) fn cooling_down(&self) -> bool {
		self.cooldown_timer > 0.0
	}
}
