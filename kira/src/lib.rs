//! # Kira
//!
//! Kira is an audio library designed to help create expressive audio
//! for games. Besides the common features you'd expect from an audio
//! library, it provides an interface for scripting audio events to
//! happen at certain times, like the tick of a metronome.
//!
//! ## Usage
//!
//! To use Kira, first create an [`AudioManager`](manager::AudioManager):
//! ```no_run
//! # use std::error::Error;
//! # use kira::manager::{AudioManager, AudioManagerSettings};
//! #
//! let mut audio_manager = AudioManager::<()>::new(AudioManagerSettings::default())?;
//! # Ok::<(), kira::AudioError>(())
//! ```
//!
//! All audio-related actions go through the [`AudioManager`](manager::AudioManager).
//!
//! ### Loading and playing a sound
//!
//! ```no_run
//! # use std::error::Error;
//! # use kira::{manager::{AudioManager, AudioManagerSettings}, sound::{Sound, SoundSettings}, instance::InstanceSettings};
//! #
//! # let mut audio_manager = AudioManager::<()>::new(AudioManagerSettings::default())?;
//! let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", SoundSettings::default())?)?;
//! audio_manager.play_sound(sound_id, InstanceSettings::default())?;
//! # Ok::<(), kira::AudioError>(())
//! ```
//!
//! ### Looping a piece of music
//!
//! ```no_run
//! # use std::error::Error;
//! #
//! # use kira::{
//! # 	instance::InstanceSettings,
//! # 	manager::AudioManager,
//! # 	sound::{Sound, SoundMetadata, SoundSettings},
//! # 	Tempo,
//! # };
//! #
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! let sound_id = audio_manager.add_sound(Sound::from_file(
//! 	"loop.ogg",
//! 	SoundSettings {
//! 		metadata: SoundMetadata {
//! 			semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
//! 		},
//! 		..Default::default()
//! 	},
//! )?)?;
//! // when the sound loops, start the loop 4 beats in
//! let loop_start = Tempo(128.0).beats_to_seconds(4.0);
//! audio_manager.play_sound(sound_id, InstanceSettings::new().loop_region(loop_start..))?;
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
//! # 	sequence::Sequence,
//! # 	sound::{Sound, SoundMetadata, SoundSettings},
//! # 	Tempo,
//! # };
//! #
//! # #[derive(Debug, Copy, Clone)]
//! # enum CustomEvent {
//! # 	KickDrum,
//! # }
//! #
//! # let mut audio_manager = AudioManager::<CustomEvent>::new(Default::default())?;
//! # let kick_drum_sound_id = audio_manager.add_sound(Sound::from_file(
//! # 	"kick.ogg",
//! # 	SoundSettings {
//! # 		metadata: SoundMetadata {
//! # 			semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
//! # 		},
//! # 		..Default::default()
//! # 	},
//! # )?)?;
//! let mut sequence = Sequence::new();
//! sequence.start_loop();
//! sequence.wait_for_interval(4.0);
//! sequence.play_sound(kick_drum_sound_id, InstanceSettings::default());
//! sequence.emit_custom_event(CustomEvent::KickDrum);
//! audio_manager.start_sequence(sequence)?;
//! // start the metronome so the sequence will have a pulse to listen for
//! audio_manager.start_metronome()?;
//! # Ok::<(), kira::AudioError>(())
//! ```

pub mod arrangement;
pub mod audio_stream;
mod command;
mod duration;
mod error;
mod event;
mod frame;
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
pub use event::Event;
pub use frame::Frame;
pub use metronome::MetronomeSettings;
pub use tempo::Tempo;
pub use value::{CachedValue, Value};
