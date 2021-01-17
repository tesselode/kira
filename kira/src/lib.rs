//! # Kira
//!
//! Kira is an audio library designed to help create expressive audio
//! for games. Besides the common features you'd expect from an audio
//! library, it provides interfaces for scripting audio events,
//! seamlessly looping complex pieces of music, smoothly changing
//! parameters, and more.
//!
//! ## Usage
//!
//! To use Kira, first create an [`AudioManager`](manager::AudioManager):
//! ```no_run
//! # use std::error::Error;
//! #
//! # use kira::manager::{AudioManager, AudioManagerSettings};
//! #
//! let mut audio_manager = AudioManager::new(AudioManagerSettings::default())?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! All audio-related actions go through the [`AudioManager`](manager::AudioManager).
//!
//! ### Loading and playing a sound
//!
//! ```no_run
//! # use std::error::Error;
//! #
//! # use kira::{
//! # 	instance::InstanceSettings,
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sound::SoundSettings,
//! # };
//! #
//! # let mut audio_manager = AudioManager::new(AudioManagerSettings::default())?;
//! let mut sound_handle = audio_manager.load_sound("sound.ogg", SoundSettings::default())?;
//! sound_handle.play(InstanceSettings::default())?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Looping a song while preserving trailing sounds
//!
//! ```no_run
//! # use std::error::Error;
//! #
//! # use kira::{
//! # 	arrangement::{Arrangement, LoopArrangementSettings},
//! # 	instance::InstanceSettings,
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sound::SoundSettings,
//! # 	Tempo,
//! # };
//! #
//! # let mut audio_manager = AudioManager::new(AudioManagerSettings::default())?;
//! let sound_handle = audio_manager.load_sound(
//! 	"loop.ogg",
//! 	SoundSettings::new().semantic_duration(Tempo(128.0).beats_to_seconds(8.0)),
//! )?;
//! let mut arrangement_handle = audio_manager.add_arrangement(Arrangement::new_loop(
//! 	&sound_handle,
//! 	LoopArrangementSettings::default(),
//! ))?;
//! arrangement_handle.play(InstanceSettings::default())?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Scripting audio sequences
//!
//! This example will play a kick drum sound every 4 beats and emit an event
//! each time:
//!
//! ```no_run
//! # use std::error::Error;
//! #
//! # use kira::{
//! # 	instance::InstanceSettings,
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	metronome::MetronomeSettings,
//! # 	sequence::{Sequence, SequenceInstanceSettings, SequenceSettings},
//! # 	sound::SoundSettings,
//! # 	Tempo,
//! # };
//! #
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! enum CustomEvent {
//! 	Kick,
//! }
//!
//! # let mut audio_manager = AudioManager::new(AudioManagerSettings::default())?;
//! let kick_sound_handle = audio_manager.load_sound("kick.wav", SoundSettings::default())?;
//! let mut metronome_handle =
//! 	audio_manager.add_metronome(MetronomeSettings::new().tempo(Tempo(150.0)))?;
//! audio_manager.start_sequence(
//! 	{
//! 		let mut sequence = Sequence::new(SequenceSettings::default());
//! 		sequence.start_loop();
//! 		sequence.play(&kick_sound_handle, InstanceSettings::default());
//! 		sequence.emit(CustomEvent::Kick);
//! 		sequence.wait(kira::Duration::Beats(1.0));
//! 		sequence
//! 	},
//! 	SequenceInstanceSettings::new().metronome(&metronome_handle),
//! )?;
//! metronome_handle.start()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod arrangement;
pub mod audio_stream;
mod command;
mod duration;
mod frame;
pub mod group;
pub mod instance;
pub mod manager;
pub mod metronome;
pub mod mixer;
pub mod parameter;
pub mod playable;
mod resource;
pub mod sequence;
pub mod sound;
mod tempo;
mod util;
mod value;

pub use duration::Duration;
pub use frame::Frame;
pub use tempo::Tempo;
pub use value::{CachedValue, Value};
