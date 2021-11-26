/*!
Tweenable values for controlling settings.

Parameters are "global" float values that many settings
can be linked to. Any setting that has the type
[`Value`](crate::value::Value) can be linked to a parameter.

## Creating and modifying parameters

To create a parameter, use [`AudioManager::add_parameter`](crate::manager::AudioManager::add_parameter)
and provide an initial value of your choice:

```no_run
use kira::manager::{AudioManager, AudioManagerSettings};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
let mut parameter = manager.add_parameter(1.0)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

At any time, you can smoothly transition the parameter to a
new value by using [`ParameterHandle::set`].

```no_run
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
default [`Tween`], which is fast enough to _feel_ instantaneous,
but still slow enough to avoid creating audio artifacts like pops
and crackles.

```no_run
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

```no_run
use std::time::Duration;

use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
	tween::Tween,
};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
let mut parameter = manager.add_parameter(1.0)?;
manager.play(kira_loaders::load(
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
[`Mapping`](crate::value::Mapping)s.

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
```ignore
Mapping {
	input_range: (0.0, 1.0),
	output_range: (20_000.0, 2_000.0),
	..Default::default()
}
```

We want the volume to be `1.0` when the parameter is set to `0.0`
and `0.5` when the parameter is set to `1.0`. To do that, we'll
use this mapping:
```ignore
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

```no_run
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
*/

mod handle;
mod parameters;

pub use handle::*;
pub use parameters::*;

use std::sync::{
	atomic::{AtomicBool, AtomicU64, Ordering},
	Arc,
};

use atomic_arena::Key;

use crate::{
	clock::Clocks,
	tween::{Tween, Tweenable},
};

type JustFinishedTween = bool;

/// A unique identifier for a parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParameterId(pub(crate) Key);

pub(crate) struct ParameterShared {
	value: AtomicU64,
	paused: AtomicBool,
	removed: AtomicBool,
}

impl ParameterShared {
	pub fn new(value: f64) -> Self {
		Self {
			value: AtomicU64::new(value.to_bits()),
			paused: AtomicBool::new(false),
			removed: AtomicBool::new(false),
		}
	}

	pub fn value(&self) -> f64 {
		f64::from_bits(self.value.load(Ordering::SeqCst))
	}

	pub fn paused(&self) -> bool {
		self.paused.load(Ordering::SeqCst)
	}

	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

pub(crate) struct Parameter {
	tweenable: Tweenable,
	paused: bool,
	shared: Arc<ParameterShared>,
}

impl Parameter {
	pub fn new(initial_value: f64) -> Self {
		Self {
			tweenable: Tweenable::new(initial_value),
			paused: false,
			shared: Arc::new(ParameterShared::new(initial_value)),
		}
	}

	pub(crate) fn shared(&self) -> Arc<ParameterShared> {
		self.shared.clone()
	}

	pub fn value(&self) -> f64 {
		self.tweenable.value()
	}

	pub fn pause(&mut self) {
		self.paused = true;
		self.shared.paused.store(true, Ordering::SeqCst);
	}

	pub fn resume(&mut self) {
		self.paused = false;
		self.shared.paused.store(false, Ordering::SeqCst);
	}

	pub fn set(&mut self, target: f64, tween: Tween) {
		self.tweenable.set(target, tween);
	}

	pub(crate) fn on_start_processing(&self) {
		self.shared
			.value
			.store(self.tweenable.value().to_bits(), Ordering::SeqCst);
	}

	pub fn update(&mut self, dt: f64, clocks: &Clocks) -> JustFinishedTween {
		if self.paused {
			return false;
		}
		self.tweenable.update(dt, clocks)
	}
}
