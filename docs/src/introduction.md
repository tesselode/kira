# Introduction

Kira is a backend-agnostic library to create expressive audio for games. It
provides parameters for smoothly adjusting properties of sounds, a flexible
mixer for applying effects to audio, and a clock system for precisely timing
audio events.

## Examples

### Playing a sound multiple times simultaneously

```rust ,no_run
# extern crate kira;
#
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::cpal::CpalBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
};

// Create an audio manager. This plays sounds and manages resources.
let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let sound_data = StaticSoundData::load("sound.ogg", StaticSoundSettings::default())?;
manager.play(sound_data.clone())?;
// After a couple seconds...
manager.play(sound_data.clone())?;
// Cloning the sound data will not use any extra memory.
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

### Gradually speeding up a sound over time

```rust ,no_run
# extern crate kira;
#
use std::time::Duration;

use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::cpal::CpalBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	tween::Tween,
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let sound_data = StaticSoundData::load("sound.ogg", StaticSoundSettings::new())?;
let mut sound = manager.play(sound_data)?;
// Start smoothly adjusting the playback rate parameter.
sound.set_playback_rate(
	2.0,
	Tween {
		duration: Duration::from_secs(3),
		..Default::default()
	},
);
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

### Playing a sound with a low-pass filter applied

This makes the audio sound muffled.

```rust ,no_run
# extern crate kira;
#
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::cpal::CpalBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	track::{
		TrackBuilder,
		effect::filter::FilterBuilder,
	},
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
// Create a mixer sub-track with a filter.
let track = manager.add_sub_track({
	let mut builder = TrackBuilder::new();
	builder.add_effect(FilterBuilder::new().cutoff(1000.0));
	builder
})?;
// Play the sound on the track.
let sound_data = StaticSoundData::load(
	"sound.ogg",
	StaticSoundSettings::new().track(&track),
)?;
manager.play(sound_data)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

### Playing sounds in time with a musical beat

```rust ,no_run
# extern crate kira;
#
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::cpal::CpalBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	ClockSpeed,
};

const TEMPO: f64 = 120.0;

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
// Create a clock that ticks 120 times per second. In this case,
// each tick is one musical beat. We can use a tick to represent any
// arbitrary amount of time.
let mut clock = manager.add_clock(ClockSpeed::TicksPerMinute(TEMPO))?;
// Play a sound 2 ticks (beats) from now.
let sound_data_1 = StaticSoundData::load(
	"sound1.ogg",
	StaticSoundSettings::new().start_time(clock.time() + 2),
)?;
manager.play(sound_data_1)?;
// Play a different sound 4 ticks (beats) from now.
let sound_data_2 = StaticSoundData::load(
	"sound2.ogg",
	StaticSoundSettings::new().start_time(clock.time() + 4),
)?;
manager.play(sound_data_2)?;
// Start the clock.
clock.start()?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```
