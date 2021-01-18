//! Combines individual sounds into larger pieces.
//!
//! `Arrangement`s are containers of `SoundClip`s, which are portions of
//! a sound that can be positioned in time, stretched, trimmed, and
//! reversed. You can play instances of an arrangement just like you would
//! play instances of a sound.
//!
//! `Arrangement`s are a lot like arrangement views in DAWs, like the
//! playlist view in FL Studio. In fact, the playlist view in FL Studio
//! will be used to illustrate the contents of `Arrangement`s in the
//! following examples.
//!
//! This image represents an arrangement that plays the same sound
//! four times: once normally, once trimmed, once stretched out,
//! and once reversed.
//!
//! ![arrangements 1](https://i.imgur.com/1p4W1Ld.png)
//!
//! ## Motivating example: seamless loops
//!
//! Oftentimes, game composers need to write pieces that loop forever.
//! These pieces may also have an intro section that plays once
//! before the main part of the song loops forever. `Instance`s allow
//! you to set a loop start point so when the playback position reaches
//! the end, it jumps back to an arbitrary point in the sound.
//!
//! The problem is this doesn't account for parts of the sound that
//! bleed into the next section. For example, at the end of an orchestral
//! piece, there may be a cymbal swell that transitions the song back
//! to the beginning of the loop. To preserve the musical timing of the
//! piece, the playback position needs to jump back to the start point
//! as soon as the last measure of music ends, which would cut off
//! the cymbal, creating a jarring sound. If the song has an intro section
//! with trailing sound, then that sound will cut in when the song
//! loops, which is also jarring.
//!
//! There's a couple possible solutions:
//! - Use a [`Sequence`](crate::sequence) to play separate
//!   intro and loop sounds at the right time. This works, but you
//!   can't reverse or change the pitch of a sequence, which you may
//!   want in some circumstances.
//! - You can edit your intro and loop audio in a specific way to create a
//!   larger piece that will seamlessly loop. This is tedious, and you have
//!   to store a larger audio as part of the game's assets.
//!
//! Arrangements let you use the latter solution without having to store
//! a larger audio file, and as you'll see, they can do the work of setting
//! up seamless loops for you.
//!
//! ### Setting up a simple loop
//!
//! Let's say we have a short drum loop with a cymbal swell at the end
//! that will seamless lead back to the beginning of the loop.
//!
//! ![arrangements 2](https://i.imgur.com/TOpa9Zq.png)
//!
//! We can set up a seamless loop by placing the same sound in an arrangement
//! twice, once with the cymbal swell preserved and once with it cut off.
//! The red region at the top shows the portion of the arrangement
//! that will be looped.
//!
//! ![arrangements 3](https://i.imgur.com/Xoti30y.png)
//!
//! When the playback position jumps back to the loop point, the trailing sound
//! from the first sound clip will seamlessly connect to the trailing
//! sound that was cut off from the second clip.
//!
//! You can set up this arrangement manually:
//!
//! ```no_run
//! # use kira::{
//! # 	arrangement::{Arrangement, ArrangementSettings, SoundClip},
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sound::{Sound, SoundSettings}, Tempo,
//! # };
//! #
//! # let mut audio_manager = AudioManager::new(AudioManagerSettings::default())?;
//! # let sound_handle = audio_manager.add_sound(Sound::from_file(
//! # 	std::env::current_dir()?.join("assets/loop.wav"),
//! # 	SoundSettings::default(),
//! # )?)?;
//! #
//! let tempo = Tempo(140.0);
//! let mut arrangement = Arrangement::new(
//! 	ArrangementSettings::new().default_loop_start(tempo.beats_to_seconds(16.0)),
//! );
//! arrangement
//! 	.add_clip(SoundClip::new(&sound_handle, 0.0))
//! 	.add_clip(
//! 		SoundClip::new(&sound_handle, tempo.beats_to_seconds(16.0))
//! 			.trim(tempo.beats_to_seconds(16.0)),
//! 	);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Or you can just use [`Arrangement::new_loop`], which will do the work for you:
//!
//! ```no_run
//! # use kira::{
//! # 	arrangement::{Arrangement, LoopArrangementSettings, SoundClip},
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sound::{Sound, SoundSettings}, Tempo,
//! # };
//! #
//! # let mut audio_manager = AudioManager::new(AudioManagerSettings::default())?;
//! let tempo = Tempo(140.0);
//! let sound_handle = audio_manager.add_sound(Sound::from_file(
//! 	std::env::current_dir()?.join("assets/loop.wav"),
//! 	SoundSettings {
//! 		semantic_duration: Some(tempo.beats_to_seconds(16.0)),
//! 		..Default::default()
//! 	},
//! )?)?;
//! let arrangement = Arrangement::new_loop(&sound_handle, LoopArrangementSettings::default());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Loops with intros
//!
//! Loops with intros can be set up in a similar way:
//!
//! ![arrangements 4](https://i.imgur.com/EM7P7Ry.png)
//!
//! For brevity, we'll just say you can use [`Arrangement::new_loop_with_intro`]
//! to create these.

mod clip;
pub mod handle;
mod id;
mod settings;

pub use clip::SoundClip;
use handle::ArrangementHandle;
pub use id::ArrangementId;
pub use settings::{ArrangementSettings, LoopArrangementSettings};

use indexmap::IndexMap;

use crate::{
	group::{groups::Groups, GroupId, GroupSet},
	mixer::TrackIndex,
	sound::{handle::SoundHandle, Sound, SoundId},
	Frame,
};

/// An arrangement of sound clips to play at specific times.
#[derive(Debug, Clone)]
pub struct Arrangement {
	id: ArrangementId,
	clips: Vec<SoundClip>,
	duration: f64,
	default_track: TrackIndex,
	cooldown: Option<f64>,
	semantic_duration: Option<f64>,
	default_loop_start: Option<f64>,
	groups: GroupSet,
	cooldown_timer: f64,
}

impl Arrangement {
	/// Creates a new, empty arrangement.
	pub fn new(settings: ArrangementSettings) -> Self {
		Self {
			id: settings.id,
			clips: vec![],
			duration: 0.0,
			default_track: settings.default_track,
			cooldown: settings.cooldown,
			semantic_duration: settings.semantic_duration,
			default_loop_start: settings.default_loop_start,
			groups: settings.groups,
			cooldown_timer: 0.0,
		}
	}

	/// Creates a new arrangement that seamlessly loops a sound.
	///
	/// If the sound has a semantic duration, it will be used to
	/// set the point where the sound loops. Any audio after the
	/// semantic end of the sound will be preserved when the sound
	/// loops.
	pub fn new_loop(sound_handle: &SoundHandle, settings: LoopArrangementSettings) -> Self {
		let duration = sound_handle
			.semantic_duration()
			.unwrap_or(sound_handle.duration());
		let mut arrangement = Self::new(ArrangementSettings {
			id: settings.id,
			default_track: settings.default_track,
			cooldown: settings.cooldown,
			semantic_duration: settings.semantic_duration,
			default_loop_start: Some(duration),
			groups: settings.groups,
		});
		arrangement
			.add_clip(SoundClip::new(sound_handle, 0.0))
			.add_clip(SoundClip::new(sound_handle, duration).trim(duration));
		arrangement
	}

	/// Creates a new arrangement that plays an intro sound, then
	/// seamlessly loops another sound.
	///
	/// If the intro has a semantic duration, it will be used to determine
	/// when the loop sound starts. If the loop sound has a semantic duration,
	/// it will be used to set the point where the sound repeats. Any audio
	/// after the semantic end of the sound will be preserved when the sound
	/// loops.
	pub fn new_loop_with_intro(
		intro_sound_handle: &SoundHandle,
		loop_sound_handle: &SoundHandle,
		settings: LoopArrangementSettings,
	) -> Self {
		let intro_duration = intro_sound_handle
			.semantic_duration()
			.unwrap_or(intro_sound_handle.duration());
		let loop_duration = loop_sound_handle
			.semantic_duration()
			.unwrap_or(loop_sound_handle.duration());
		let mut arrangement = Self::new(ArrangementSettings {
			id: settings.id,
			default_track: settings.default_track,
			cooldown: settings.cooldown,
			semantic_duration: settings.semantic_duration,
			default_loop_start: Some(intro_duration + loop_duration),
			groups: settings.groups,
		});
		arrangement
			.add_clip(SoundClip::new(intro_sound_handle, 0.0))
			.add_clip(SoundClip::new(loop_sound_handle, intro_duration))
			.add_clip(
				SoundClip::new(loop_sound_handle, intro_duration + loop_duration)
					.trim(loop_duration),
			);
		arrangement
	}

	/// Adds a sound clip to the arrangement.
	pub fn add_clip(&mut self, clip: SoundClip) -> &mut Self {
		self.duration = self.duration.max(clip.clip_time_range.1);
		self.clips.push(clip);
		self
	}

	/// Gets the unique identifier for this arrangement.
	pub fn id(&self) -> ArrangementId {
		self.id
	}

	/// Gets the duration of the arrangement.
	///
	/// The duration is always the end of the last playing sound clip.
	pub fn duration(&self) -> f64 {
		self.duration
	}

	/// Gets the default track instances of this arrangement will play on.
	pub fn default_track(&self) -> TrackIndex {
		self.default_track
	}

	/// Gets the groups this arrangement belongs to.
	pub fn groups(&self) -> &GroupSet {
		&self.groups
	}

	/// Gets the "musical length" of the arrangement (if there is one).
	pub fn semantic_duration(&self) -> Option<f64> {
		self.semantic_duration
	}

	/// Returns the default time (in seconds) instances
	/// of this arrangement will loop back to when they reach
	/// the end.
	pub fn default_loop_start(&self) -> Option<f64> {
		self.default_loop_start
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
		if let Some(cooldown) = self.cooldown {
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

	/// Returns if this arrangement is in the group with the given ID.
	pub(crate) fn is_in_group(&self, id: GroupId, all_groups: &Groups) -> bool {
		self.groups.has_ancestor(id, all_groups)
	}
}
