# Clocks

## Creating clocks

Clocks can be used to set the start times of sounds and tweens. To create a
clock, use `AudioManager::add_clock`.

```rust ,no_run
# extern crate kira;
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::cpal::CpalBackend,
	},
	ClockSpeed,
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let mut clock = manager.add_clock(ClockSpeed::SecondsPerTick(1.0))?;
clock.start()?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

You can specify the speed of the clock as seconds per tick, ticks per second, or
ticks per minute.

Clocks are stopped when you first create them, so be sure to explicitly call
`ClockHandle::start` when you want the clock to start ticking.

## Starting sounds on clock ticks

Static sounds (and streaming sounds from the
[`kira-streaming`](https://crates.io/crates/kira-streaming) crate) can be set to
only start playing when a clock has ticked a certain number of times. You can
configure this using `StaticSoundSettings::start_time`.

```rust ,no_run
# extern crate kira;
use kira::{
	clock::ClockTime,
	manager::{
		AudioManager, AudioManagerSettings,
		backend::cpal::CpalBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	StartTime, ClockSpeed,
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let mut clock = manager.add_clock(ClockSpeed::SecondsPerTick(1.0))?;
manager.play(StaticSoundData::from_file(
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
# extern crate kira;
# use kira::{
# 	clock::ClockTime,
# 	manager::{
# 	 	AudioManager, AudioManagerSettings,
# 		backend::cpal::CpalBackend,
# 	},
# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
# 	ClockSpeed, StartTime,
# };
#
# let mut manager = AudioManager::<CpalBackend>::new(
# 	AudioManagerSettings::default(),
# )?;
# let mut clock = manager.add_clock(ClockSpeed::SecondsPerTick(1.0))?;
manager.play(StaticSoundData::from_file(
	"sound.ogg",
	StaticSoundSettings::new().start_time(ClockTime {
		clock: clock.id(),
		ticks: 4,
	}),
)?)?;
# clock.start()?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

As an even shorter hand, you can use `ClockHandle::time` to get the clock's
current `ClockTime`, and then add to it to get a time in the future:

```rust ,no_run
# extern crate kira;
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::cpal::CpalBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	ClockSpeed,
};

# let mut manager = AudioManager::<CpalBackend>::new(
# 	AudioManagerSettings::default(),
# )?;
# let mut clock = manager.add_clock(ClockSpeed::SecondsPerTick(1.0))?;
manager.play(StaticSoundData::from_file(
	"sound.ogg",
	StaticSoundSettings::new().start_time(clock.time() + 4),
)?)?;
# clock.start()?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

## Starting tweens on clock ticks

You can also use clocks to set the start time of tweens. In this example, we set
the playback rate of a sound to start tweening when a clock reaches its third
tick.

```rust ,no_run
# extern crate kira;
use std::time::Duration;

use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::cpal::CpalBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	tween::Tween,
	ClockSpeed, StartTime,
};

let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
let mut clock = manager.add_clock(ClockSpeed::SecondsPerTick(1.0))?;
let mut sound = manager.play(StaticSoundData::from_file(
	"sound.ogg",
	StaticSoundSettings::default(),
)?)?;
sound.set_playback_rate(
	0.5,
	Tween {
		start_time: StartTime::ClockTime(clock.time() + 3),
		duration: Duration::from_secs(2),
		..Default::default()
	},
)?;
clock.start()?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```
