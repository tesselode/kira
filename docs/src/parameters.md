# Parameters

Parameters are "global" float values that many settings
can be linked to. Any setting that has the type `Value`
can be linked to a parameter.

## Creating and modifying parameters

To create a parameter, use `AudioManager::add_parameter`
and provide an initial value of your choice:

```rust ,no_run
use kira::manager::{AudioManager, AudioManagerSettings};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
let mut parameter = manager.add_parameter(1.0)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

At any time, you can smoothly transition the parameter to a
new value by using `ParameterHandle::set`.

```rust ,no_run
use std::time::Duration;

use kira::{
	manager::{AudioManager, AudioManagerSettings},
	tween::Tween,
};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
let mut parameter = manager.add_parameter(1.0)?;
parameter.set(
	2.0,
	Tween {
		duration: Duration::from_secs(2),
		..Default::default()
	},
)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())

```

If you'd like to instantaneously set the value, you can use the
default `Tween`, which is fast enough to _feel_ instantaneous,
but still slow enough to avoid creating audio artifacts like pops
and crackles.

```rust ,no_run
# use std::time::Duration;
#
# use kira::{
# 	manager::{AudioManager, AudioManagerSettings},
# 	tween::Tween,
# };
# use kira_cpal::CpalBackend;
#
# let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
# let mut parameter = manager.add_parameter(1.0)?;
parameter.set(2.0, Tween::default())?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

## Using parameters

Many settings can be linked to parameters. In this example, the
playback rate of a sound is linked directly to a parameter.
It will start out at normal speed, and speed up to 2x speed
over the course of 2 seconds.

```rust ,no_run
use std::time::Duration;

use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
	tween::Tween,
};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
let mut parameter = manager.add_parameter(1.0)?;
manager.play(kira_symphonia::load(
	"sound.ogg",
	StaticSoundSettings::new().playback_rate(&parameter),
)?)?;
parameter.set(
	2.0,
	Tween {
		duration: Duration::from_secs(2),
		..Default::default()
	},
)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

## Custom mappings

Multiple settings can be linked to the same parameter. In this
case, we may want different settings to have different
relationships to the parameter. We can do this using
`Mapping`s.

Say that when a character in a game enters water, we want
certain sounds to become more muffled and quieter. To accomplish
this effect, we need to change two settings:
- A filter cutoff in **Hz**, which controls how muffled the sound is
- A volume level, which is a factor of the sound's normal volume

We'll start by arbitrarily deciding that our parameter should
stay in a range of `0.0` to `1.0`, where `0.0` is not underwater
at all and `1.0` is fully submerged.

We want the filter cutoff to be at `20_000.0` Hz when the parameter
is set to `0.0` and `2_000.0` Hz when the parameter is set to `1.0`.
To do that, we'll use this mapping:
```rust ,ignore
Mapping {
	input_range: (0.0, 1.0),
	output_range: (20_000.0, 2_000.0),
	..Default::default()
}
```

We want the volume to be `1.0` when the parameter is set to `0.0`
and `0.5` when the parameter is set to `1.0`. To do that, we'll
use this mapping:
```rust ,ignore
Mapping {
	input_range: (0.0, 1.0),
	output_range: (1.0, 0.5),
	..Default::default()
}
```

Note that in both of these cases, the second value of the
output range is less than the first value. This is perfectly
valid and means that as the parameter increases, the resulting
value will decrease.

The full code would look something like this:

```rust ,no_run
use std::time::Duration;

use kira::{
	manager::{AudioManager, AudioManagerSettings},
	track::TrackSettings,
	tween::Tween,
	value::{Mapping, Value},
};
use kira_cpal::CpalBackend;
use kira_effects::filter::{Filter, FilterSettings};

let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
let mut underwater_parameter = manager.add_parameter(0.0)?;
manager.add_sub_track(
	TrackSettings::new()
		.volume(Value::Parameter {
			id: underwater_parameter.id(),
			mapping: Mapping {
				input_range: (0.0, 1.0),
				output_range: (1.0, 0.5),
				..Default::default()
			},
		})
		.with_effect(Filter::new(FilterSettings::new().cutoff(
			Value::Parameter {
				id: underwater_parameter.id(),
				mapping: Mapping {
					input_range: (0.0, 1.0),
					output_range: (20_000.0, 2_000.0),
					..Default::default()
				},
			},
		))),
)?;
underwater_parameter.set(
	1.0,
	Tween {
		duration: Duration::from_secs(2),
		..Default::default()
	},
)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```
