/*!
Organizes and applies effects to audio.

Kira has an internal mixer which works like a real-life mixing console. Sounds
can be played on "tracks", which are individual streams of audio that can
optionally have effects that modify the audio.

## Creating and using tracks

The mixer has a "main" track by default, and you can add any number of
sub-tracks. To add a sub-track, use `AudioManager::add_sub_track`.

```no_run
# use std::error::Error;
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::DefaultBackend,
	},
	track::TrackBuilder,
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let track = manager.add_sub_track(TrackBuilder::default())?;
# Result::<(), Box<dyn Error>>::Ok(())
```

You can configure what track a sound will play on by modifying its settings.
This example uses `StaticSoundSettings`, but `StreamingSoundSettings` provides
the same option.

```no_run
# use std::error::Error;
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::DefaultBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	track::TrackBuilder,
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let track = manager.add_sub_track(TrackBuilder::default())?;
manager.play(StaticSoundData::from_file(
	"sound.ogg",
	StaticSoundSettings::new().output_destination(&track),
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

```no_run
# use std::error::Error;
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::DefaultBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	track::{
		TrackBuilder,
		effect::filter::FilterBuilder,
	},
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let track = manager.add_sub_track(
	TrackBuilder::new()
		.with_effect(FilterBuilder::new().cutoff(1000.0))
)?;
manager.play(StaticSoundData::from_file(
	"sound.ogg",
	StaticSoundSettings::new().output_destination(&track),
)?)?;
# Result::<(), Box<dyn Error>>::Ok(())
```

`TrackBuilder:add_effect` returns a handle that can be used to modify the effect
after the track has been created.

```no_run
# use kira::{
# 	manager::{
#     AudioManager, AudioManagerSettings,
#     backend::DefaultBackend,
#   },
# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
# 	track::{effect::filter::FilterBuilder, TrackBuilder},
# 	tween::Tween,
# };
# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let mut filter;
let track = manager.add_sub_track({
	let mut builder = TrackBuilder::new();
	filter = builder.add_effect(FilterBuilder::new().cutoff(1000.0));
	builder
})?;
filter.set_cutoff(4000.0, Tween::default());
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
.        ┌──────────┐
.        │Main track│
.        └─▲──────▲─┘
.          │      │
.          │      │
.     ┌────┴─┐   ┌┴────┐
.     │Sounds│   │Music│
.     └──▲───┘   └─────┘
.        │
. ┌──────┴──────┐
. │Player sounds│
. └─────────────┘
```

We can set up the `sounds` and `player_sounds` hierarchy using `TrackRoutes`.

```no_run
# use std::error::Error;
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::DefaultBackend,
	},
	track::{TrackRoutes, TrackBuilder},
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
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
.         ┌──────────┐
.    ┌────►Main track◄───────┐
.    │    └─▲────────┘       │
.    │      │                │
.    │      │            ┌───┴──┐
.    │ ┌────┼────────────►Reverb│
.    │ │    │            └──▲───┘
.    │ │    │               │
.    │ │    │               │
. ┌──┴─┴─┐  │   ┌────────┐  │
. │Player│  └───┤Ambience├──┘
. └──────┘      └────────┘
```

Here's what this looks like in practice:

```no_run
# use std::error::Error;
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::DefaultBackend,
	},
	track::{
		TrackRoutes, TrackBuilder,
		effect::reverb::ReverbBuilder,
	},
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
// 1.
let reverb = manager.add_sub_track({
	let mut builder = TrackBuilder::new();
	builder.add_effect(ReverbBuilder::new().mix(1.0));
	builder
})?;
// 2.
let player = manager.add_sub_track(
	TrackBuilder::new().routes(TrackRoutes::new().with_route(&reverb, 0.25)),
);
// 3.
let ambience = manager.add_sub_track(
	TrackBuilder::new().routes(TrackRoutes::new().with_route(&reverb, 0.5)),
);
# Result::<(), Box<dyn Error>>::Ok(())
```

1. We create the `reverb` track with a `Reverb` effect. We set the `mix` to `1.0`
so that only the reverb signal is output from this track. We don't need any of
the dry signal to come out of this track, since the `player` and `ambience`
tracks will already be outputting their dry signal to the main track.

2. We create the `player` track with two routes:

	- The route to the main track with 100% volume. We don't have to set this one
	explicitly because `TrackRoutes::new()` adds that route by default.
	- The route to the `reverb` track with 25% volume.

3. The `ambience` track is set up the same way, except the route to the `reverb`
track has 50% volume, giving us more reverb for these sounds.
*/

mod builder;
pub mod effect;
mod handle;
mod routes;

#[cfg(test)]
mod test;

pub use builder::*;
pub use handle::*;
pub use routes::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use crate::arena::Key;

use crate::{
	clock::clock_info::ClockInfoProvider,
	command::{CommandReader, ValueChangeCommand},
	dsp::Frame,
	modulator::value_provider::ModulatorValueProvider,
	tween::Parameter,
	Volume,
};

use self::effect::Effect;

/// A unique identifier for a mixer sub-track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubTrackId(pub(crate) Key);

/// A unique identifier for a track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrackId {
	/// The main mixer track.
	Main,
	/// A sub-track.
	Sub(SubTrackId),
}

impl From<SubTrackId> for TrackId {
	fn from(id: SubTrackId) -> Self {
		Self::Sub(id)
	}
}

impl From<&TrackHandle> for TrackId {
	fn from(handle: &TrackHandle) -> Self {
		handle.id()
	}
}

pub(crate) struct TrackShared {
	removed: AtomicBool,
}

impl TrackShared {
	pub fn new() -> Self {
		Self {
			removed: AtomicBool::new(false),
		}
	}

	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

pub(crate) struct Track {
	shared: Arc<TrackShared>,
	volume: Parameter<Volume>,
	volume_change_command_reader: CommandReader<ValueChangeCommand<Volume>>,
	routes: Vec<Route>,
	effects: Vec<Box<dyn Effect>>,
	input: Frame,
}

impl Track {
	pub fn init_effects(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.init(sample_rate);
		}
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.on_change_sample_rate(sample_rate);
		}
	}

	pub fn shared(&self) -> Arc<TrackShared> {
		self.shared.clone()
	}

	pub fn routes_mut(&mut self) -> &mut Vec<Route> {
		&mut self.routes
	}

	pub fn add_input(&mut self, input: Frame) {
		self.input += input;
	}

	pub fn on_start_processing(&mut self) {
		self.volume
			.read_commands(&mut self.volume_change_command_reader);
		for Route {
			volume,
			volume_change_command_reader,
			..
		} in &mut self.routes
		{
			volume.read_commands(volume_change_command_reader);
		}
		for effect in &mut self.effects {
			effect.on_start_processing();
		}
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		self.volume
			.update(dt, clock_info_provider, modulator_value_provider);
		for Route { volume, .. } in &mut self.routes {
			volume.update(dt, clock_info_provider, modulator_value_provider);
		}
		let mut output = std::mem::replace(&mut self.input, Frame::ZERO);
		for effect in &mut self.effects {
			output = effect.process(output, dt, clock_info_provider, modulator_value_provider);
		}
		output * self.volume.value().as_amplitude()
	}
}

pub(crate) struct Route {
	pub(crate) destination: TrackId,
	pub(crate) volume: Parameter<Volume>,
	pub(crate) volume_change_command_reader: CommandReader<ValueChangeCommand<Volume>>,
}
