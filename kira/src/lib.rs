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
//! # use kira::manager::{AudioManager, AudioManagerSettings};
//! #
//! let mut audio_manager = AudioManager::new(AudioManagerSettings::default())?;
//! # Ok::<(), kira::AudioError>(())
//! ```
//!
//! All audio-related actions go through the [`AudioManager`](manager::AudioManager).
//!
//! ### Loading and playing a sound
//!
//! ```no_run
//! # use std::error::Error;
//! # use kira::{
//! # 	manager::{AudioManager, AudioManagerSettings},
//! # 	sound::Sound,
//! # 	playable::PlayableSettings,
//! # 	instance::InstanceSettings,
//! # };
//! #
//! # let mut audio_manager = AudioManager::new(AudioManagerSettings::default())?;
//! let sound_id = audio_manager.load_sound("loop.ogg", PlayableSettings::default())?;
//! audio_manager.play(sound_id, InstanceSettings::default())?;
//! # Ok::<(), kira::AudioError>(())
//! ```
//!
//! ### Looping a song while preserving trailing sounds
//!
//! ```no_run
//! # use std::error::Error;
//! #
//! # use kira::{
//! # 	arrangement::{Arrangement, LoopArrangementSettings}, manager::{AudioManager, AudioManagerSettings},
//! # 	playable::PlayableSettings, sound::Sound, instance::InstanceSettings,
//! # 	Tempo,
//! # };
//! #
//! # let mut audio_manager = AudioManager::new(Default::default())?;
//! let sound_id = audio_manager.load_sound(
//! 	std::env::current_dir()?.join("assets/loop.wav"),
//! 	PlayableSettings {
//! 		semantic_duration: Some(Tempo(140.0).beats_to_seconds(16.0)),
//! 		..Default::default()
//! 	},
//! )?;
//! let arrangement = Arrangement::new_loop(sound_id, LoopArrangementSettings::default());
//! let arrangement_id = audio_manager.add_arrangement(arrangement)?;
//! audio_manager.play(arrangement_id, InstanceSettings::default())?;
//! # Ok::<(), kira::AudioError>(())
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
//! # 	manager::AudioManager,
//! # 	sequence::{Sequence, SequenceSettings, SequenceInstanceSettings},
//! # 	sound::Sound,
//! # 	playable::PlayableSettings,
//! # 	Tempo,
//! # };
//! #
//! #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
//! enum CustomEvent {
//! 	KickDrum,
//! }
//! #
//! # let mut audio_manager = AudioManager::new(Default::default())?;
//! # let kick_drum_sound_id = audio_manager.add_sound(Sound::from_file(
//! # 	"kick.ogg",
//! # 	PlayableSettings {
//! # 		semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
//! # 		..Default::default()
//! # 	},
//! # )?)?;
//! let mut sequence = Sequence::<CustomEvent>::new(SequenceSettings::default());
//! sequence.start_loop();
//! sequence.wait_for_interval(4.0);
//! sequence.play(kick_drum_sound_id, InstanceSettings::default());
//! sequence.emit(CustomEvent::KickDrum);
//! let (id, mut event_receiver) = audio_manager.start_sequence(sequence, SequenceInstanceSettings::default())?;
//! // start the metronome so the sequence will have a pulse to listen for
//! audio_manager.start_metronome()?;
//! // pop events
//! while let Some(event) = event_receiver.pop() {
//! 	println!("{:?}", event);
//! }
//! # Ok::<(), kira::AudioError>(())
//! ```

/*
TODO: figure out if remove functions (remove_parameter, remove_sub_track,
etc.) should be on the audio manager or on the corresponding handles.
the benefit of having those functions on the handles is that it
declutters the audio manager. however, if one piece of code passes
a handle to a track/parameter/etc. to another, you may not want to allow
removing the thing you have a handle to.
*/

pub mod arrangement;
pub mod audio_stream;
mod command;
mod duration;
mod error;
mod frame;
pub mod group;
pub mod instance;
pub mod manager;
mod metronome;
pub mod mixer;
pub mod parameter;
pub mod playable;
pub mod sequence;
pub mod sound;
mod tempo;
mod util;
mod value;

pub use duration::Duration;
pub use error::{AudioError, AudioResult};
pub use frame::Frame;
pub use metronome::MetronomeSettings;
pub use tempo::Tempo;
pub use value::{CachedValue, Value};
