# Introduction

Kira is a backend-agnostic library to create expressive audio for games. Besides
the common sound playback features, it provides parameters for smoothly
adjusting properties of sounds, a flexible mixer for applying effects to audio,
and a clock system for precisely timing audio events.

## Examples

### Playing a sound multiple times simultaneously

```rust ,no_run
# extern crate kira;
# extern crate kira_cpal;
# extern crate kira_symphonia;
#
use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
};
use kira_cpal::CpalBackend;

// Create an audio manager. This plays sounds and manages resources.
let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
let sound_data = kira_symphonia::load("sound.ogg", StaticSoundSettings::default())?;
manager.play(sound_data.clone())?;
// After a couple seconds...
manager.play(sound_data.clone())?;
// Cloning the sound data will not use any extra memory.
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

### Gradually speeding up a sound over time

```rust ,no_run
# extern crate kira;
# extern crate kira_cpal;
# extern crate kira_symphonia;
#
use std::time::Duration;

use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
	tween::Tween,
};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
// Create a parameter for the playback rate.
let mut parameter = manager.add_parameter(1.0)?;
let sound_data = kira_symphonia::load(
	"sound.ogg",
	// Link this sound's playback rate to the parameter we created.
	StaticSoundSettings::new().playback_rate(&parameter),
)?;
manager.play(sound_data)?;
// Start smoothly adjusting the playback rate parameter.
parameter.set(
	2.0,
	Tween {
		duration: Duration::from_secs(3),
		..Default::default()
	},
)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

### Playing a sound with a low-pass filter applied

This makes the audio sound muffled.

```rust ,no_run
# extern crate kira;
# extern crate kira_cpal;
# extern crate kira_effects;
# extern crate kira_symphonia;
#
use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
	track::TrackSettings,
};
use kira_cpal::CpalBackend;
use kira_effects::filter::{Filter, FilterSettings};

let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
// Create a mixer sub-track with a filter.
let filter = Filter::new(FilterSettings::new().cutoff(1000.0));
let track = manager.add_sub_track(TrackSettings::new().with_effect(filter))?;
// Play the sound on the track.
let sound_data =
	kira_symphonia::load("sound.ogg", StaticSoundSettings::new().track(&track))?;
manager.play(sound_data)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

### Playing sounds in time with a musical beat

```rust ,no_run
# extern crate kira;
# extern crate kira_cpal;
# extern crate kira_symphonia;
#
use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
};
use kira_cpal::CpalBackend;

const TEMPO: f64 = 120.0;

let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
// Create a clock that ticks every 60.0 / TEMPO seconds. In this case,
// each tick is one beat. Of course, we can use a tick to represent
// any arbitrary amount of time.
let mut clock = manager.add_clock(60.0 / TEMPO)?;
// Play a sound 2 ticks (beats) from now.
let sound_data_1 = kira_symphonia::load(
	"sound1.ogg",
	StaticSoundSettings::new().start_time(clock.time() + 2),
)?;
manager.play(sound_data_1)?;
// Play a different sound 4 ticks (beats) from now.
let sound_data_2 = kira_symphonia::load(
	"sound2.ogg",
	StaticSoundSettings::new().start_time(clock.time() + 4),
)?;
manager.play(sound_data_2)?;
// Start the clock.
clock.start()?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```
