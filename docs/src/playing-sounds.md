# Playing Sounds

The main `kira` crate does not come with any functionality for loading audio
from files. For that, you should use
[`kira-loaders`](https://crates.io/crates/kira-loaders).

`kira_loaders::load` returns a `StaticSoundData` that you can pass to
`AudioManager::play` to play the sound.

```rust ,no_run
use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(
	CpalBackend::new()?,
	AudioManagerSettings::default(),
)?;
let sound_data = kira_loaders::load("sound.ogg", StaticSoundSettings::new())?;
manager.play(sound_data)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

If you want to play a sound multiple times, keep a copy of the `StaticSoundData`
around and clone it each time you pass it to `AudioManager::play`.

```rust ,no_run
use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(
	CpalBackend::new()?,
	AudioManagerSettings::default(),
)?;
let sound_data = kira_loaders::load("sound.ogg", StaticSoundSettings::new())?;
manager.play(sound_data.clone())?;
manager.play(sound_data.clone())?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

Cloning a `StaticSoundData` is cheap, so it's perfectly fine to do this.

`AudioManager::play` returns a handle to the sound that you can use to query
information about the sound or modify it.

```rust ,no_run
use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::{PlaybackState, StaticSoundSettings},
	tween::Tween,
};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(
	CpalBackend::new()?,
	AudioManagerSettings::default(),
)?;
let sound_data = kira_loaders::load("sound.ogg", StaticSoundSettings::new())?;
let mut sound = manager.play(sound_data)?;
if sound.state() == PlaybackState::Playing {
	sound.stop(Tween::default())?;
}
```

## Streaming sounds

The previous examples all used `kira_symphonia::load`, which loads the entire
sound into memory. This is good for shorter sounds, but for longer sounds this
can have a heavy memory footprint. In those cases, you may want to use
`kira_symphonia::stream`, which will read data from disk in realtime as the
sound is playing.

There are some disadvantages to using streaming sounds:

- Streaming sounds require more CPU power.
- There may be a longer delay between when you call `AudioManager::play` and
  when the sound actually starts playing.
- Seeking the sound may also have a longer delay.
- If the file cannot be read from the disk fast enough, there will be hiccups in
  the sound playback. (This will not affect other sounds, though.)
- Backwards playback is not supported.
- `StreamingSoundData` cannot be cloned.
