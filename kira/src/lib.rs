//! # Kira
//!
//! Kira is an audio library designed to help create expressive audio
//! for games. Besides the common features you'd expect from an audio
//! library, it provides an interface for scripting audio events to
//! happen at certain times, like the tick of a metronome.
//!
//! ## Usage
//!
//! To use Kira, first create an `AudioManager`:
//! ```no_run
//! # use std::error::Error;
//! # use kira::manager::AudioManager;
//! #
//! let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! # Ok::<(), kira::KiraError>(())
//! ```
//!
//! All audio-related actions go through the `AudioManager`.
//!
//! ### Loading and playing a sound
//!
//! ```no_run
//! # use std::error::Error;
//! # use kira::manager::AudioManager;
//! #
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! let sound_id = audio_manager.load_sound("loop.ogg", Default::default())?;
//! audio_manager.play_sound(sound_id, Default::default())?;
//! # Ok::<(), kira::KiraError>(())
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
//! # 	sound::{SoundMetadata, SoundSettings},
//! # 	Tempo,
//! # };
//! #
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! let sound_id = audio_manager.load_sound(
//! 	"loop.ogg",
//! 	SoundSettings {
//! 		metadata: SoundMetadata {
//! 			// mark the sound as being 16 beats long. even if the sound has
//! 			// a reverb tail at the end, it will loop after 16 beats (with
//! 			// the reverb tail preserved)
//! 			semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
//! 		},
//! 		..Default::default()
//! 	},
//! )?;
//! // when the sound loops, start the loop 4 beats in
//! let loop_start = Tempo(128.0).beats_to_seconds(4.0);
//! audio_manager.play_sound(sound_id, InstanceSettings::new().loop_region(loop_start..))?;
//! # Ok::<(), kira::KiraError>(())
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
//! # 	sound::{SoundMetadata, SoundSettings},
//! # 	Tempo,
//! # };
//! #
//! # #[derive(Debug, Copy, Clone)]
//! # enum CustomEvent {
//! # 	KickDrum,
//! # }
//! #
//! # let mut audio_manager = AudioManager::<CustomEvent>::new(Default::default())?;
//! # let kick_drum_sound_id = audio_manager.load_sound(
//! # 	"loop.ogg",
//! # 	SoundSettings {
//! # 		metadata: SoundMetadata {
//! # 			semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
//! # 		},
//! # 		..Default::default()
//! # 	},
//! # )?;
//! let mut sequence = Sequence::new();
//! sequence.start_loop();
//! sequence.wait_for_interval(4.0);
//! sequence.play_sound(kick_drum_sound_id, Default::default());
//! sequence.emit_custom_event(CustomEvent::KickDrum);
//! audio_manager.start_sequence(sequence)?;
//! // start the metronome so the sequence will have a pulse to listen for
//! audio_manager.start_metronome()?;
//! # Ok::<(), kira::KiraError>(())
//! ```

mod command;
mod duration;
mod error;
pub mod instance;
pub mod manager;
mod metronome;
mod parameter;
pub mod sequence;
pub mod sound;
mod stereo_sample;
mod tempo;
pub mod track;
mod tween;
mod value;

pub use duration::Duration;
pub use error::{KiraError, KiraResult};
pub use metronome::MetronomeSettings;
pub use parameter::ParameterId;
pub use tempo::Tempo;
pub use tween::Tween;
pub use value::Value;
