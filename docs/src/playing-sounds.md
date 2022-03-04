# Playing Sounds

To start using Kira, create an `AudioManager`.

```rust ,no_run
# extern crate kira;
use kira::manager::{
	AudioManager, AudioManagerSettings,
	backend::cpal::CpalBackend,
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

The `AudioManager` allows you to interact with the audio context from gameplay
code. `AudioManager`s can play anything that implements the `SoundData` trait,
such as `StaticSoundData` or `StreamingSoundData`.

```rust ,no_run
# extern crate kira;
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::cpal::CpalBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let sound_data = StaticSoundData::load("sound.ogg", StaticSoundSettings::new())?;
manager.play(sound_data)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

If you want to play a sound multiple times, keep a copy of the `StaticSoundData`
around and clone it each time you pass it to `AudioManager::play`.

```rust ,no_run
# extern crate kira;
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::cpal::CpalBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let sound_data = StaticSoundData::load("sound.ogg", StaticSoundSettings::new())?;
manager.play(sound_data.clone())?;
manager.play(sound_data.clone())?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

Cloning a `StaticSoundData` is cheap, so it's perfectly fine to do this.

`StreamingSoundData` cannot be cloned, so you will have to create a new one each
time you want to play a sound.

## Modifying playing sounds

`AudioManager::play` returns a handle to the sound that you can use to query
information about the sound or modify it.

```rust ,no_run
# extern crate kira;
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::cpal::CpalBackend,
	},
	sound::static_sound::{PlaybackState, StaticSoundData, StaticSoundSettings},
	tween::Tween,
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let sound_data = StaticSoundData::load("sound.ogg", StaticSoundSettings::new())?;
let mut sound = manager.play(sound_data)?;
if sound.state() == PlaybackState::Playing {
	sound.stop(Tween::default())?;
}
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

Many parameters of sounds, like volume and playback rate, can be smoothly
transitioned to other values.

```rust ,no_run
# extern crate kira;
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
sound.set_volume(
	0.5,
	Tween {
		duration: Duration::from_secs(2),
		..Default::default()
	},
)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

Some property setters allow you to set the value in different units. For
example, volumes can be set in decibels:

```rust ,no_run
# extern crate kira;
# use std::time::Duration;
# use kira::{
# 	manager::{
# 		AudioManager, AudioManagerSettings,
# 		backend::cpal::CpalBackend,
# 	},
# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
# 	tween::Tween,
# 	Volume,
# };
# let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
# let sound_data = StaticSoundData::load("sound.ogg", StaticSoundSettings::new())?;
# let mut sound = manager.play(sound_data)?;
sound.set_volume(
	Volume::Decibels(-3.0),
	Tween {
		duration: Duration::from_secs(2),
		..Default::default()
	},
)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

If you want to change a property instantaneously, use the default `Tween`. It's
fast enough to sound instantaneous, but slow enough to avoid audio artifacts.
