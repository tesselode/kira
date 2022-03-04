# The Mixer

Kira has an internal mixer which works like a real-life mixing console. Sounds
can be played on "tracks", which are individual streams of audio that can
optionally have effects that modify the audio.

## Creating and using tracks

The mixer has a "main" track by default, and you can add any number of
sub-tracks. To add a sub-track, use `AudioManager::add_sub_track`.

```rust ,no_run
# extern crate kira;
# use std::error::Error;
use kira::{
    manager::{
        AudioManager, AudioManagerSettings,
        backend::cpal::CpalBackend,
    },
    track::TrackBuilder,
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let track = manager.add_sub_track(TrackBuilder::default())?;
# Result::<(), Box<dyn Error>>::Ok(())
```

You can configure what track a sound will play on by modifying its settings.
This example uses `StaticSoundSettings`, but `StreamingSoundSettings` provides
the same option.

```rust ,no_run
# extern crate kira;
# use std::error::Error;
use kira::{
	manager::{
        AudioManager, AudioManagerSettings,
        backend::cpal::CpalBackend,
    },
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
	track::TrackBuilder,
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let track = manager.add_sub_track(TrackBuilder::default())?;
manager.play(StaticSoundData::load(
    "sound.ogg",
    StaticSoundSettings::new().track(&track),
)?)?;
# Result::<(), Box<dyn Error>>::Ok(())
```

You can set the volume and panning of a track using `TrackHandle::set_volume`
and `TrackHandle::set_panning`, respectively. The volume and panning settings
will affect all sounds being played on the track.

## Effects

You can add effects to the track when creating it using
`TrackBuilder::add_effect`. All sounds that are played on that track will have
the effects applied sequentially.

In this example, we'll use the `Filter` effect, which in the low pass mode will
remove high frequencies from sounds, making them sound muffled.

```rust ,no_run
# extern crate kira;
# use std::error::Error;
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
let track = manager.add_sub_track({
    let mut builder = TrackBuilder::new();
    builder.add_effect(FilterBuilder::new().cutoff(1000.0));
    builder
})?;
manager.play(StaticSoundData::load(
    "sound.ogg",
    StaticSoundSettings::new().track(&track),
)?)?;
# Result::<(), Box<dyn Error>>::Ok(())
```

`TrackBuilder:add_effect` returns a handle that can be used to modify the effect
after the track has been created.

```rust ,no_run
# extern crate kira;
# use kira::{
# 	manager::{
#     AudioManager, AudioManagerSettings,
#     backend::cpal::CpalBackend,
#   },
# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
# 	track::{effect::filter::FilterBuilder, TrackBuilder},
# 	tween::Tween,
# };
# let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let mut filter;
let track = manager.add_sub_track({
	let mut builder = TrackBuilder::new();
	filter = builder.add_effect(FilterBuilder::new().cutoff(1000.0));
	builder
})?;
filter.set_cutoff(4000.0, Tween::default())?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

## Track routing

By default, the output of all sub-tracks will be fed into the input of the main
mixer track without any volume change. It can be useful to customize this
behavior.

Let's say we want to be able to control the volume level of gameplay sounds
separately from music. We may also want to apply effects to gameplay sounds that
come from the player specifically.

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

We can set up the `sounds` and `player_sounds` hierarchy using `TrackRoutes`.

```rust ,no_run
# extern crate kira;
# use std::error::Error;
use kira::{
	manager::{
        AudioManager, AudioManagerSettings,
        backend::cpal::CpalBackend,
    },
	track::{TrackRoutes, TrackBuilder},
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let sounds = manager.add_sub_track(TrackBuilder::default())?;
let player_sounds = manager.add_sub_track(
	TrackBuilder::new().routes(TrackRoutes::parent(&sounds)),
)?;
# Result::<(), Box<dyn Error>>::Ok(())
```

The default `TrackRoutes` has a single route to the main mixer track.
`TrackRoutes::parent` will instead create a single route to the track of your
choosing.

You can also have one track feed its audio into multiple other tracks. This can
be useful for sharing effects between tracks.

For example, let's say we have our sounds split up into player sounds and
ambience. This game takes place in a vast cave, so we want all of the sounds to
have a reverb effect. We want the ambience to have more reverb than the player
sounds so that it feels farther away.

We could put separate reverb effects on both the `player` and `ambience` tracks.
Since both the player and the ambient sounds are in the same cave, we'll use the
same settings for both reverb effects, but we'll increase the `mix` setting for
the ambience, since ambient sounds are supposed to have more reverb. This has
some downsides, however:

- Since most of the settings are supposed to be the same between the two tracks,
  if we want to change the reverb settings, we have to change them in two
  different places.
- We have two separate reverb effects running, which has a higher CPU cost than
  if we just had one.

A better alternative would be to make a separate reverb track that both the
`player` and `ambience` tracks are routed to.

```text
        ┌──────────┐
   ┌────►Main track◄───────┐
   │    └─▲────────┘       │
   │      │                │
   │      │            ┌───┴──┐
   │ ┌────┼────────────►Reverb│
   │ │    │            └──▲───┘
   │ │    │               │
   │ │    │               │
┌──┴─┴─┐  │   ┌────────┐  │
│Player│  └───┤Ambience├──┘
└──────┘      └────────┘
```

Here's what this looks like in practice:

```rust ,no_run
# extern crate kira;
# use std::error::Error;
use kira::{
	manager::{
        AudioManager, AudioManagerSettings,
        backend::cpal::CpalBackend,
    },
	track::{
        TrackRoutes, TrackBuilder,
        effect::reverb::ReverbBuilder,
    },
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let reverb = manager.add_sub_track({
    let mut builder = TrackBuilder::new();
    builder.add_effect(ReverbBuilder::new().mix(1.0));
    builder
})?;
let player = manager.add_sub_track(
    TrackBuilder::new().routes(TrackRoutes::new().with_route(&reverb, 0.25)),
);
let ambience = manager.add_sub_track(
    TrackBuilder::new().routes(TrackRoutes::new().with_route(&reverb, 0.5)),
);
# Result::<(), Box<dyn Error>>::Ok(())
```

Let's look at this one step at a time:

```rust ,ignore
let reverb = manager.add_sub_track({
    let mut builder = TrackBuilder::new();
    builder.add_effect(ReverbBuilder::new().mix(1.0));
    builder
})?;
```

We create the `reverb` track with a `Reverb` effect. We set the `mix` to `1.0`
so that only the reverb signal is output from this track. We don't need any of
the dry signal to come out of this track, since the `player` and `ambience`
tracks will already be outputting their dry signal to the main track.

```rust ,ignore
let player = manager.add_sub_track(
    TrackBuilder::new().routes(TrackRoutes::new().with_route(&reverb, 0.25)),
);
```

We create the `player` track with two routes:

- The route to the main track with 100% volume. We don't have to set this one
  explicitly because `TrackRoutes::new()` adds that route by default.
- The route to the `reverb` track with 25% volume.

```rust ,ignore
let ambience = manager.add_sub_track(
    TrackBuilder::new().routes(TrackRoutes::new().with_route(&reverb, 0.5)),
);
```

The `ambience` track is set up the same way, except the route to the `reverb`
track has 50% volume, giving us more reverb for these sounds.
