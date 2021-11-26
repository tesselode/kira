/*!
Organizes and applies effects to audio.

Kira has an internal mixer which works like a real-life
mixing console. Sounds can be played on "tracks", which are
individual streams of audio that can optionally have effects
that modify the audio.

## Creating and using tracks

The mixer has a "main" track by default, and you can add
any number of sub-tracks. To add a sub-track, use
[`AudioManager::add_sub_track`](crate::manager::AudioManager::add_sub_track).

```no_run
use std::error::Error;

use kira::{manager::AudioManager, track::TrackSettings};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(CpalBackend::new()?, Default::default())?;
let track = manager.add_sub_track(TrackSettings::default())?;
# Result::<(), Box<dyn Error>>::Ok(())
```

You can configure what track a sound will play on by modifying
its settings. This example uses
[`StaticSoundSettings`](crate::sound::static_sound::StaticSoundSettings),
but the streaming sound interface from
[`kira-loaders`](https://crates.io/crates/kira-loaders) provides
the same option.

```no_run
use std::error::Error;

use kira::{manager::AudioManager, sound::static_sound::StaticSoundSettings, track::TrackSettings};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(CpalBackend::new()?, Default::default())?;
let track = manager.add_sub_track(TrackSettings::default())?;
manager.play(kira_loaders::load(
	"sound.ogg",
	StaticSoundSettings::new().track(&track),
)?)?;
# Result::<(), Box<dyn Error>>::Ok(())
```

You can add effects to the track when creating it using
[`TrackSettings::with_effect`]. All sounds that are played
on that track will have the effects applied sequentially.

In this example, we'll use the `Filter` effect from
[`kira-effects`](https://crates.io/crates/kira-effects), which
in the low pass mode will remove high frequencies from sounds,
making them sound muffled.

```no_run
use std::error::Error;

use kira::{manager::AudioManager, sound::static_sound::StaticSoundSettings, track::TrackSettings};
use kira_cpal::CpalBackend;
use kira_effects::filter::{Filter, FilterSettings};

let mut manager = AudioManager::new(CpalBackend::new()?, Default::default())?;
let track = manager.add_sub_track(
	TrackSettings::new().with_effect(Filter::new(FilterSettings::new().cutoff(1000.0))),
)?;
manager.play(kira_loaders::load(
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
[`TrackRoutes`].
*/

mod effect;
mod handle;
mod routes;
mod settings;

pub use effect::*;
pub use handle::*;
pub use routes::*;
pub use settings::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use atomic_arena::Key;

use crate::{
	dsp::Frame,
	manager::backend::context::Context,
	parameter::Parameters,
	value::{CachedValue, Value},
};

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
	volume: CachedValue,
	panning: CachedValue,
	routes: Vec<(TrackId, CachedValue)>,
	effects: Vec<Box<dyn Effect>>,
	input: Frame,
}

impl Track {
	pub fn new(mut settings: TrackSettings, context: &Arc<Context>) -> Self {
		for effect in &mut settings.effects {
			effect.init(context.sample_rate());
		}
		Self {
			shared: Arc::new(TrackShared::new()),
			volume: CachedValue::new(.., settings.volume, 1.0),
			panning: CachedValue::new(0.0..=1.0, settings.panning, 0.5),
			routes: settings.routes.into_vec(),
			effects: settings.effects,
			input: Frame::ZERO,
		}
	}

	pub fn shared(&self) -> Arc<TrackShared> {
		self.shared.clone()
	}

	pub fn routes_mut(&mut self) -> &mut Vec<(TrackId, CachedValue)> {
		&mut self.routes
	}

	pub fn set_volume(&mut self, volume: Value) {
		self.volume.set(volume);
	}

	pub fn set_panning(&mut self, panning: Value) {
		self.panning.set(panning);
	}

	pub fn add_input(&mut self, input: Frame) {
		self.input += input;
	}

	pub fn process(&mut self, dt: f64, parameters: &Parameters) -> Frame {
		self.volume.update(parameters);
		self.panning.update(parameters);
		for (_, amount) in &mut self.routes {
			amount.update(parameters);
		}
		let mut output = std::mem::replace(&mut self.input, Frame::ZERO);
		for effect in &mut self.effects {
			output = effect.process(output, dt, parameters);
		}
		output *= self.volume.get() as f32;
		output = output.panned(self.panning.get() as f32);
		output
	}
}
