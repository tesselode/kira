# The Mixer

Kira has an internal mixer which works like a real-life
mixing console. Sounds can be played on "tracks", which are
individual streams of audio that can optionally have effects
that modify the audio.

## Creating and using tracks

The mixer has a "main" track by default, and you can add
any number of sub-tracks. To add a sub-track, use
`AudioManager::add_sub_track`.

```rust ,no_run
use std::error::Error;

use kira::{manager::AudioManager, track::TrackSettings};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(CpalBackend::new()?, Default::default())?;
let track = manager.add_sub_track(TrackSettings::default())?;
# Result::<(), Box<dyn Error>>::Ok(())
```

You can configure what track a sound will play on by modifying
its settings. This example uses `StaticSoundSettings`, but the
streaming sound interface from [`kira-symphonia`](https://crates.io/crates/kira-symphonia)
provides the same option.

```rust ,no_run
use std::error::Error;

use kira::{manager::AudioManager, sound::static_sound::StaticSoundSettings, track::TrackSettings};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(CpalBackend::new()?, Default::default())?;
let track = manager.add_sub_track(TrackSettings::default())?;
manager.play(kira_symphonia::load(
    "sound.ogg",
    StaticSoundSettings::new().track(&track),
)?)?;
# Result::<(), Box<dyn Error>>::Ok(())
```

You can add effects to the track when creating it using
`TrackSettings::with_effect`. All sounds that are played
on that track will have the effects applied sequentially.

In this example, we'll use the `Filter` effect from
[`kira-effects`](https://crates.io/crates/kira-effects), which
in the low pass mode will remove high frequencies from sounds,
making them sound muffled.

```rust ,no_run
use std::error::Error;

use kira::{manager::AudioManager, sound::static_sound::StaticSoundSettings, track::TrackSettings};
use kira_cpal::CpalBackend;
use kira_effects::filter::{Filter, FilterSettings};

let mut manager = AudioManager::new(CpalBackend::new()?, Default::default())?;
let track = manager.add_sub_track(
    TrackSettings::new().with_effect(Filter::new(FilterSettings::new().cutoff(1000.0))),
)?;
manager.play(kira_symphonia::load(
    "sound.ogg",
    StaticSoundSettings::new().track(&track),
)?)?;
# Result::<(), Box<dyn Error>>::Ok(())
```

## Track routing

By default, the output of all sub-tracks will be fed into the input
of the main mixer track without any volume change. It can be useful
to customize this behavior.

Let's say we want to be able to control the volume level of
gameplay sounds separately from music. We may also want to apply
effects to gameplay sounds that come from the player specifically.

We'll end up with a hierarchy like this:

```text
       ┌──────────┐
       │Main track│
       └─▲──────▲─┘
         │      │
         │      │
    ┌────┴─┐   ┌┴────┐
    │Sounds│   │Music│
    └──▲───┘   └─────┘
       │
┌──────┴──────┐
│Player sounds│
└─────────────┘
```

We can set up the `sounds` and `player_sounds` hierarchy using
`TrackRoutes`.
