/*!
# Conductor

**Conductor** (working title) is an audio library for games with a focus on dynamic music.
It allows for playing audio with precise timing and syncing gameplay events to music.

This library is in *very* early development. I'm making this public so people can give
feedback on the design. Of course, the full vision of this library hasn't come to fruition,
but I'm hoping people can catch issues and suggest improvements before I get too
deep into development.

## Basic usage

Create a Project to hold your audio data:
```no_run
# use conductor::{
# 	instance::InstanceSettings,
# 	manager::{AudioManager, AudioManagerSettings},
# 	project::Project,
# };
# use std::error::Error;
#
# fn main() -> Result<(), Box<dyn Error>> {
let mut project = Project::new();
let sound_id = project.load_sound(
	&std::env::current_dir()
		.unwrap()
		.join("assets/test_loop.ogg"),
)?;
# let mut audio_manager = AudioManager::new(project, AudioManagerSettings::default())?;
# audio_manager.play_sound(sound_id, InstanceSettings::default())?;
# loop {}
# }
```

Create an AudioManager to start the audio thread:
```no_run
# use conductor::{
# 	instance::InstanceSettings,
# 	manager::{AudioManager, AudioManagerSettings},
# 	project::Project,
# };
# use std::error::Error;
#
# fn main() -> Result<(), Box<dyn Error>> {
# let mut project = Project::new();
# let sound_id = project.load_sound(
# 	&std::env::current_dir()
# 		.unwrap()
# 		.join("assets/test_loop.ogg"),
# )?;
let mut audio_manager = AudioManager::new(project, AudioManagerSettings::default())?;
# audio_manager.play_sound(sound_id, InstanceSettings::default())?;
# loop {}
# }
```

Play a sound:
```no_run
# use conductor::{
# 	instance::InstanceSettings,
# 	manager::{AudioManager, AudioManagerSettings},
# 	project::Project,
# };
# use std::error::Error;
#
# fn main() -> Result<(), Box<dyn Error>> {
# let mut project = Project::new();
# let sound_id = project.load_sound(
# 	&std::env::current_dir()
# 		.unwrap()
# 		.join("assets/test_loop.ogg"),
# )?;
# let mut audio_manager = AudioManager::new(project, AudioManagerSettings::default())?;
audio_manager.play_sound(sound_id, InstanceSettings::default())?;
# loop {}
# }
```

## Roadmap

Currently implemented features:
- Loading .ogg files
- Basic audio playback
- Sequences synced to metronomes

Planned features:
- Loading .wav, .mp3, and .flac files
- Audio tracks and effects
- Parameters for smoothly changing various values

Maybe in the future:
- C bindings?
*/

mod command;
pub mod error;
pub mod instance;
pub mod manager;
pub mod metronome;
mod parameter;
pub mod project;
pub mod sequence;
pub mod sound;
mod stereo_sample;
pub mod time;
pub mod tween;

pub use manager::AudioManager;
pub use project::Project;
