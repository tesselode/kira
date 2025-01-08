/*!
Organizes and applies effects to audio.

Kira has an internal mixer which works like a real-life mixing console. Sounds
can be played on "tracks", which are individual streams of audio that can
optionally have effects that modify the audio.

Tracks can also be spatialized, which gives them a position in a 3D space
relative to a [listener](crate::listener). The distance from the listener
can be used to drive settings on effects on that track.

## Creating and using tracks

The mixer has a "main" track by default, and you can add any number of
sub-tracks. To add a sub-track, use
[`AudioManager::add_sub_track`](crate::AudioManager::add_sub_track).

```no_run
# use std::error::Error;
use kira::{
	AudioManager, AudioManagerSettings, DefaultBackend,
	track::TrackBuilder,
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let mut track = manager.add_sub_track(TrackBuilder::default())?;
# Result::<(), Box<dyn Error>>::Ok(())
```

To play a sound on the track, use [`TrackHandle::play`].

```no_run
# use std::error::Error;
# use kira::{
# 	AudioManager, AudioManagerSettings, backend::DefaultBackend,
# 	track::TrackBuilder,
# };
use kira::sound::static_sound::StaticSoundData;

# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
# let mut track = manager.add_sub_track(TrackBuilder::default())?;
track.play(StaticSoundData::from_file("sound.ogg")?)?;
# Result::<(), Box<dyn Error>>::Ok(())
```

Tracks can themselves have sub-tracks. You can create them using [`TrackHandle::add_sub_track`].

You can set the volume of a track using [`TrackHandle::set_volume`]. The volume
setting will affect all sounds being played on the track as well as all child tracks.

You can pause all sounds (and child tracks) of a track using [`TrackHandle::pause`]
and resume them using [`TrackHandle::resume`] or [`TrackHandle::resume_at`].

## Effects

You can add effects to the track when creating it using
[`TrackBuilder::with_effect`]. All sounds that are played on that track will have
the effects applied sequentially.

In this example, we'll use the [filter](crate::effect::filter) effect, which in the
low pass mode will remove high frequencies from sounds, making them sound muffled.

```no_run
# use std::error::Error;
use kira::{
	AudioManager, AudioManagerSettings, DefaultBackend,
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	track::TrackBuilder,
	effect::filter::FilterBuilder,
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let mut track = manager.add_sub_track(
	TrackBuilder::new()
		.with_effect(FilterBuilder::new().cutoff(1000.0))
)?;
track.play(StaticSoundData::from_file("sound.ogg")?)?;
# Result::<(), Box<dyn Error>>::Ok(())
```

[`TrackBuilder::add_effect`] returns a handle that can be used to modify the effect
after the track has been created.

```no_run
# use std::error::Error;
# use kira::{
#   AudioManager, AudioManagerSettings, DefaultBackend,
#   sound::static_sound::{StaticSoundData, StaticSoundSettings},
#   track::TrackBuilder,
#   effect::filter::FilterBuilder,
#   Tween,
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

## Send tracks

Sometimes, you may want to share effects across multiple tracks. For example, let's say
we have a game with a player and an enemy that both make sounds, so we have a "player"
track and an "enemy" track. We want sounds for both of these tracks to have reverb
applied. We could add a separate reverb effect to both the player and enemy track,
but there's a couple reasons this isn't an optimal solution:

- Since the player and enemy are in the same space, they should be using the same
  reverb settings. But if we have two reverb effects, we're duplicating the settings.
- Having more effects takes more CPU time, and it's wasteful in this case because
  both reverb effects are doing the same thing.

This is where send tracks come in handy. Send tracks are non-hierarchical mixer tracks
which you can't play sounds on directly - instead, regular mixer tracks can have their
output sent to the input of one or more send tracks. The output of the send tracks
is then sent to the main mixer track.

In the following example, we'll set up mixer tracks to have the following flow of audio:

```text
┌──────┐         ┌──────────┐
│      ├─────────►          │
│Player├───────┐ │Main track│
│      │ ┌─────┼─►          │
└──────┘ │     │ └─────▲────┘
┌──────┐ │ ┌───▼──┐    │
│      ├─┘ │      │    │
│Enemy ├───►Reverb├────┘
│      │   │(send)│
└──────┘   └──────┘
```

To start, we'll create the reverb send with
[`AudioManager::add_send_track`](crate::AudioManager::add_send_track):

```no_run
# use std::error::Error;
use kira::{
	effect::reverb::ReverbBuilder,
	AudioManager, AudioManagerSettings, DefaultBackend,
	track::SendTrackBuilder,
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let reverb_send = manager
	.add_send_track(SendTrackBuilder::new().with_effect(ReverbBuilder::new().mix(1.0)))?;
# Result::<(), Box<dyn Error>>::Ok(())
```

Note that we set the mix of the reverb to `1.0`, meaning only the reverberations will be output,
not the dry signal. This is important because the player and enemy tracks will already
be outputting the dry signal. If the reverb effect was also outputting the dry signal, the
overall volume of the sound would be louder than we want.

Next, we'll create the player and enemy tracks and route them to the reverb send using
[`TrackBuilder::with_send`]:

```no_run
# use std::error::Error;
use kira::track::TrackBuilder;
# use kira::{
# 	effect::reverb::ReverbBuilder,
# 	AudioManager, AudioManagerSettings, DefaultBackend,
# 	track::SendTrackBuilder,
# };

# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
# let reverb_send = manager
# 	.add_send_track(SendTrackBuilder::new().with_effect(ReverbBuilder::new().mix(1.0)))?;
let player_track = manager.add_sub_track(TrackBuilder::new().with_send(&reverb_send, -6.0))?;
let enemy_track = manager.add_sub_track(TrackBuilder::new().with_send(&reverb_send, -12.0))?;
# Result::<(), Box<dyn Error>>::Ok(())
```

We can use the second argument of `with_send` to change the volume of the track before sending
it to the send track. This allows the player and enemy to have different amounts of reverb
without having to instantiate two separate effects.

## Spatial tracks

Oftentimes, it’s useful to give sounds a location in a 3D (or 2D) space and play back those sounds
from the perspective of a character’s ears located somewhere else in that space. For example, as a
player character gets closer to a waterfall, you may want the sound of the waterfall to get louder.

We can use **spatial tracks** as the sound source and [listener](crate::listener)s for the
character's ears.

First, let's create a listener using
[`AudioManager::add_listener`](crate::AudioManager::add_listener):

```no_run
# use std::error::Error;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let listener = manager.add_listener(glam::Vec3::ZERO, glam::Quat::IDENTITY)?;
# Result::<(), Box<dyn Error>>::Ok(())
```

This example uses `glam`, but you can use any math library that has interoperability
with `mint`.

Next, we'll create a spatial track that's linked to the listener using
[`AudioManager::add_spatial_sub_track`](crate::AudioManager::add_spatial_sub_track):

```no_run
# use std::error::Error;
# use kira::{
# 	AudioManager, AudioManagerSettings, DefaultBackend,
# };
use kira::track::SpatialTrackBuilder;

# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
# let listener = manager.add_listener(glam::Vec3::ZERO, glam::Quat::IDENTITY)?;
let spatial_track = manager.add_spatial_sub_track(
	&listener,
	glam::vec3(0.0, 0.0, 10.0), // track position
	SpatialTrackBuilder::new(),
)?;
# Result::<(), Box<dyn Error>>::Ok(())
```

Now any sounds played on the spatial track will automatically have the following behaviors:

- Attenuation: sounds will get quieter the farther away they are from the listener
- Spatialization: sounds will be panned left or right depending on their direction from
  the listener

We can customize or disable these behaviors using the methods on [`SpatialTrackBuilder`].

We can also map any effect setting that uses `Value` to the distance between the spatial
track and the listener using `Value::FromListenerDistance`. One common use for this is to
change the amount of reverb a sound has based on distance:

```no_run
# use std::error::Error;
use kira::{
	effect::reverb::ReverbBuilder,
	Easing, Mapping, Value,
	Mix,
};
# use kira::{
# 	AudioManager, AudioManagerSettings, DefaultBackend,
# 	track::SpatialTrackBuilder,
# };

# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
# let listener = manager.add_listener(glam::Vec3::ZERO, glam::Quat::IDENTITY)?;
manager.add_spatial_sub_track(
	&listener,
	glam::vec3(0.0, 0.0, 10.0),
	SpatialTrackBuilder::new().with_effect(ReverbBuilder::new().mix(
		Value::FromListenerDistance(
			Mapping {
				input_range: (0.0, 100.0),
				output_range: (Mix::DRY, Mix::WET),
				easing: Easing::Linear,
			},
		),
	)),
)?;
# Result::<(), Box<dyn Error>>::Ok(())
```
*/

mod main;
mod send;
mod sub;

pub use main::*;
pub use send::*;
pub use sub::*;

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use crate::sound::PlaybackState;

#[derive(Debug)]
pub(crate) struct TrackShared {
	state: AtomicU8,
	removed: AtomicBool,
}

impl TrackShared {
	pub fn new() -> Self {
		Self {
			state: AtomicU8::new(TrackPlaybackState::Playing as u8),
			removed: AtomicBool::new(false),
		}
	}

	pub fn state(&self) -> TrackPlaybackState {
		match self.state.load(Ordering::SeqCst) {
			0 => TrackPlaybackState::Playing,
			1 => TrackPlaybackState::Pausing,
			2 => TrackPlaybackState::Paused,
			3 => TrackPlaybackState::WaitingToResume,
			4 => TrackPlaybackState::Resuming,
			_ => panic!("Invalid playback state"),
		}
	}

	pub fn set_state(&self, playback_state: PlaybackState) {
		self.state.store(playback_state as u8, Ordering::SeqCst);
	}

	#[must_use]
	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

/// The playback state of a mixer sub-track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TrackPlaybackState {
	/// The track is playing normally.
	Playing,
	/// The track is fading out, and when the fade-out
	/// is finished, playback will pause.
	Pausing,
	/// Playback is paused.
	Paused,
	/// The track is paused, but is schedule to resume in the future.
	WaitingToResume,
	/// The track is fading back in after being previously paused.
	Resuming,
}

impl TrackPlaybackState {
	/// Whether the track is outputting audio given
	/// its current playback state.
	pub fn is_advancing(self) -> bool {
		match self {
			TrackPlaybackState::Playing => true,
			TrackPlaybackState::Pausing => true,
			TrackPlaybackState::Paused => false,
			TrackPlaybackState::WaitingToResume => false,
			TrackPlaybackState::Resuming => true,
		}
	}
}

impl From<PlaybackState> for TrackPlaybackState {
	fn from(value: PlaybackState) -> Self {
		match value {
			PlaybackState::Playing => Self::Playing,
			PlaybackState::Pausing => Self::Pausing,
			PlaybackState::Paused => Self::Paused,
			PlaybackState::WaitingToResume => Self::WaitingToResume,
			PlaybackState::Resuming => Self::Resuming,
			PlaybackState::Stopping => unreachable!(),
			PlaybackState::Stopped => unreachable!(),
		}
	}
}
