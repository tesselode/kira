# Clocks

## Creating clocks

Clocks can be used to set the start times of sounds and
tweens. To create a clock, use `AudioManager::add_clock`.

```rust ,no_run
use kira::manager::{AudioManager, AudioManagerSettings};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
let mut clock = manager.add_clock(0.5)?;
clock.start()?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

When you create a clock, you have to specify the **interval**
in seconds, which determines how much time there is between
clock ticks. In this example, the clock has an interval of half
a second, which means it ticks twice per second.

Clocks are stopped when you first create them, so be sure to
explicitly call `ClockHandle::start` when you want the clock
to start ticking.

## Starting sounds on clock ticks

Static sounds (and streaming sounds from the
[`kira-streaming`](https://crates.io/crates/kira-streaming) crate)
can be set to only start playing when a clock has ticked
a certain number of times. You can configure this using
`StaticSoundSettings::start_time`.

```rust ,no_run
use kira::{
	clock::ClockTime,
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
	StartTime,
};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
let mut clock = manager.add_clock(0.5)?;
manager.play(kira_symphonia::load(
	"sound.ogg",
	StaticSoundSettings::new().start_time(StartTime::ClockTime(ClockTime {
		clock: clock.id(),
		ticks: 4,
	})),
)?)?;
clock.start()?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

As a shorthand, you can pass the `ClockTime` directly into
`StaticSoundSettings::start_time`.

```rust ,no_run
# use kira::{
# 	clock::ClockTime,
# 	manager::{AudioManager, AudioManagerSettings},
# 	sound::static_sound::StaticSoundSettings,
# 	StartTime,
# };
# use kira_cpal::CpalBackend;
#
# let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
# let mut clock = manager.add_clock(0.5)?;
manager.play(kira_symphonia::load(
	"sound.ogg",
	StaticSoundSettings::new().start_time(ClockTime {
		clock: clock.id(),
		ticks: 4,
	}),
)?)?;
# clock.start()?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

As an even shorter hand, you can use `ClockHandle::time` to get
the clock's current `ClockTime`, and then add to it to get
a time in the future:

```rust ,no_run
use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundSettings,
};
use kira_cpal::CpalBackend;

# let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
# let mut clock = manager.add_clock(0.5)?;
manager.play(kira_symphonia::load(
	"sound.ogg",
	StaticSoundSettings::new().start_time(clock.time() + 4),
)?)?;
# clock.start()?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

## Starting tweens on clock ticks

You can also use clocks to set the start time of tweens. In this
example, we set a parameter to start tweening when a clock reaches
a certain tick:

```rust ,no_run
use std::time::Duration;

use kira::{
	manager::{AudioManager, AudioManagerSettings},
	tween::Tween,
	StartTime,
};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(CpalBackend::new()?, AudioManagerSettings::default())?;
let mut clock = manager.add_clock(0.5)?;
let mut parameter = manager.add_parameter(1.0)?;
parameter.set(
	2.0,
	Tween {
		duration: Duration::from_secs(2),
		start_time: StartTime::ClockTime(clock.time() + 3),
		..Default::default()
	},
)?;
clock.start()?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```
