# `kira-loaders`

`kira-loaders` provides support for loading and streaming sounds from audio
files in Kira.

## Examples

### Loading a sound into memory all at once

```rust
use kira::{
	manager::{backend::MockBackend, AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
};

const SAMPLE_RATE: u32 = 48_000;
let mut manager = AudioManager::new(
	MockBackend::new(SAMPLE_RATE),
	AudioManagerSettings::default(),
)
.unwrap();
manager.play(kira_loaders::load(
	"sound.ogg",
	StaticSoundSettings::default(),
)?)?;
```

### Streaming a sound from disk

```rust
use kira::manager::{backend::MockBackend, AudioManager, AudioManagerSettings};
use kira_loaders::StreamingSoundSettings;

const SAMPLE_RATE: u32 = 48_000;
let mut manager = AudioManager::new(
	MockBackend::new(SAMPLE_RATE),
	AudioManagerSettings::default(),
)
.unwrap();
manager.play(kira_loaders::stream(
	"sound.ogg",
	StreamingSoundSettings::default(),
)?)?;
```

## License

This project is licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE)
- MIT license (LICENSE-MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `kira-loaders` by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
