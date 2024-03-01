/*!
Global values that parameters (like volume and playback rate) can be linked to.

Any type that implements [`ModulatorBuilder`] can be added to an audio manager by
using [`AudioManager::add_modulator`](crate::manager::AudioManager::add_modulator).

If needed, you can create custom modulators by implementing the [`ModulatorBuilder`]
and [`Modulator`] traits.

# Why modulators?

Many properties of things in Kira, like the volumes of sounds, can be smoothly
transitioned from one value to another without the use of modulators. Modulators
become handy when:

- You want to control multiple properties of objects in lockstep
- You need to change a property in a way that's more complicated than a simple
transition

# Tweener example

Let's say we have a music track with two layers that play simultaneously:
drums and piano. When the player character enters water, we want the music
to sound "underwater", so we'll fade out the drums and make the piano sound
more muffled using a low-pass filter.

For this situation, a [`tweener`] is appropriate. Tweeners hold a value
that doesn't change until we tell it to, and the value can be smoothly
transitioned.

The tweener is an input value that will generate multiple output values:
the drums volume and piano filter frequency. When the tweener is set to
`0.0`, that represents that the player is not underwater, and when it's
`1.0`, the player is submerged. (These are arbitrary values.)

| Tweener value | Drums volume | Piano filter frequency |
|---------------|--------------|------------------------|
| 0.0           | 1.0          | 20,000 Hz              |
| 1.0           | 0.0          | 500 Hz                 |

First, let's create the tweener:

```no_run
use kira::{
	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
	modulator::tweener::TweenerBuilder,
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let mut tweener = manager.add_modulator(TweenerBuilder { initial_value: 0.0 })?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

Next, we'll create a mixer track with a low-pass filter effect. The piano will play
on this track so we can make it sound more or less muffled.

```no_run
use kira::{
# 	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
# 	modulator::tweener::TweenerBuilder,
	track::{TrackBuilder, effect::filter::FilterBuilder},
	tween::{ModulatorMapping, Value},
};

# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
# let mut tweener = manager.add_modulator(TweenerBuilder { initial_value: 0.0 })?;
let filter_builder = FilterBuilder::new()
	.cutoff(Value::from_modulator(&tweener, ModulatorMapping {
		input_range: (0.0, 1.0),
		output_range: (20_000.0, 500.0),
		..Default::default()
	}));
let piano_track = manager.add_sub_track(TrackBuilder::new().with_effect(filter_builder))?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

We use a `ModulatorMapping` to map the input values of the tweener to the output values
of the filter cutoff frequency.

Finally, we'll play the sounds:

```no_run
use kira::{
# 	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
# 	modulator::tweener::TweenerBuilder,
# 	track::{TrackBuilder, effect::filter::FilterBuilder},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
# 	tween::{ModulatorMapping, Value},
	Volume,
};

# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
# let mut tweener = manager.add_modulator(TweenerBuilder { initial_value: 0.0 })?;
# let filter_builder = FilterBuilder::new()
# 	.cutoff(Value::from_modulator(&tweener, ModulatorMapping {
# 		input_range: (0.0, 1.0),
# 		output_range: (20_000.0, 500.0),
# 		..Default::default()
# 	}));
# let piano_track = manager.add_sub_track(TrackBuilder::new().with_effect(filter_builder))?;
let piano_settings = StaticSoundSettings::new().output_destination(&piano_track);
manager.play(StaticSoundData::from_file("piano.ogg", piano_settings)?)?;
let drums_settings = StaticSoundSettings::new()
	.volume(Value::from_modulator(&tweener, ModulatorMapping {
		input_range: (0.0, 1.0),
		output_range: (Volume::Amplitude(1.0), Volume::Amplitude(0.0)),
		..Default::default()
	}));
manager.play(StaticSoundData::from_file("drums.ogg", drums_settings)?)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

Notice how we also use a `ModulatorMapping` to map the input range of the tweener to the
output values of the sound volume.

Once the player goes underwater, we can smoothly transition the tweener's value from
`0.0` to `1.0`, which will automatically change the drum volume and piano filter frequency.

```no_run
# use kira::{
# 	manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
# 	modulator::tweener::TweenerBuilder,
# 	track::{TrackBuilder, effect::filter::FilterBuilder},
# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
# 	tween::{ModulatorMapping, Value},
# 	Volume,
# };
use kira::tween::Tween;
use std::time::Duration;

# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
# let mut tweener = manager.add_modulator(TweenerBuilder { initial_value: 0.0 })?;
# let filter_builder = FilterBuilder::new()
# 	.cutoff(Value::from_modulator(&tweener, ModulatorMapping {
# 		input_range: (0.0, 1.0),
# 		output_range: (20_000.0, 500.0),
# 		..Default::default()
# 	}));
# let piano_track = manager.add_sub_track(TrackBuilder::new().with_effect(filter_builder))?;
# manager.play(
# 	StaticSoundData::from_file("piano.ogg", StaticSoundSettings::new().output_destination(&piano_track))?
# )?;
# let drums_settings = StaticSoundSettings::new()
# 	.volume(Value::from_modulator(&tweener, ModulatorMapping {
# 		input_range: (0.0, 1.0),
# 		output_range: (Volume::Amplitude(1.0), Volume::Amplitude(0.0)),
# 		..Default::default()
# 	}));
# manager.play(StaticSoundData::from_file("drums.ogg", drums_settings)?)?;
tweener.set(1.0, Tween {
	duration: Duration::from_secs(3),
	..Default::default()
})?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

*/

pub mod lfo;
pub mod tweener;
pub mod value_provider;

use crate::arena::Key;

use crate::clock::clock_info::ClockInfoProvider;

use self::value_provider::ModulatorValueProvider;

/// Configures a modulator.
pub trait ModulatorBuilder {
	/// Allows the user to control the modulator from gameplay code.
	type Handle;

	/// Creates the modulator and a handle to the modulator.
	fn build(self, id: ModulatorId) -> (Box<dyn Modulator>, Self::Handle);
}

/// Produces a stream of values that a parameter can be linked to.
pub trait Modulator: Send {
	/// Called whenever a new batch of audio samples is requested by the backend.
	///
	/// This is a good place to put code that needs to run fairly frequently,
	/// but not for every single audio sample.
	fn on_start_processing(&mut self) {}

	/// Updates the modulator.
	///
	/// `dt` is the time that's elapsed since the previous round of
	/// processing (in seconds).
	fn update(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	);

	/// Returns the current output of the modulator.
	fn value(&self) -> f64;

	/// Whether the modulator can be removed from the audio context.
	fn finished(&self) -> bool;
}

/// A unique identifier for a modulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModulatorId(pub(crate) Key);
