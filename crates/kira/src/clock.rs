/*!
Precise timing for audio events.

Clocks can be used to set the start times of sounds and tweens. To create a
clock, use [`AudioManager::add_clock`](crate::manager::AudioManager::add_clock).

```no_run
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::DefaultBackend,
	},
	clock::ClockSpeed,
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
let mut clock = manager.add_clock(ClockSpeed::SecondsPerTick(1.0))?;
clock.start()?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```

You can specify the speed of the clock as seconds per tick, ticks per second, or
ticks per minute.

Clocks are stopped when you first create them, so be sure to explicitly call
[`ClockHandle::start`] when you want the clock to start ticking.

## Starting sounds on clock ticks

Sounds can be set to only start playing when a clock has ticked a certain
number of times. You can configure this using
[`StaticSoundSettings::start_time`](crate::sound::static_sound::StaticSoundSettings::start_time)
or [`StreamingSoundSettings::start_time`](crate::sound::streaming::StreamingSoundSettings::start_time).

```no_run
use kira::{
	clock::{ClockTime, ClockSpeed},
	manager::{
		AudioManager, AudioManagerSettings,
		backend::DefaultBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	StartTime,
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
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

As a shorthand, you can pass the [`ClockTime`] directly into
the `start_time` function.

```no_run
# use kira::{
# 	clock::{ClockTime, ClockSpeed},
# 	manager::{
# 	 	AudioManager, AudioManagerSettings,
# 		backend::DefaultBackend,
# 	},
# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
# 	StartTime,
# };
#
# let mut manager = AudioManager::<DefaultBackend>::new(
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

As an even shorter hand, you can use [`ClockHandle::time`] to get the clock's
current [`ClockTime`], and then add to it to get a time in the future:

```no_run
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::DefaultBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	clock::ClockSpeed,
};

# let mut manager = AudioManager::<DefaultBackend>::new(
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

```no_run
use std::time::Duration;

use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::DefaultBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	tween::Tween,
	clock::ClockSpeed,
	StartTime,
};

let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
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
*/

pub mod clock_info;
mod clock_speed;
mod handle;
mod time;

#[cfg(test)]
mod test;

pub use clock_speed::*;
pub use handle::*;
pub use time::*;

use std::sync::{
	atomic::{AtomicBool, AtomicU64, Ordering},
	Arc,
};

use crate::arena::Key;

use crate::{
	modulator::value_provider::ModulatorValueProvider,
	tween::{Parameter, Tween, Value},
};

use self::clock_info::ClockInfoProvider;

/// A unique identifier for a clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockId(pub(crate) Key);

pub(crate) struct ClockShared {
	ticking: AtomicBool,
	ticks: AtomicU64,
	fractional_position: AtomicU64,
	removed: AtomicBool,
}

impl ClockShared {
	pub fn new() -> Self {
		Self {
			ticking: AtomicBool::new(false),
			ticks: AtomicU64::new(0),
			fractional_position: AtomicU64::new(0.0f64.to_bits()),
			removed: AtomicBool::new(false),
		}
	}

	pub fn ticking(&self) -> bool {
		self.ticking.load(Ordering::SeqCst)
	}

	pub fn ticks(&self) -> u64 {
		self.ticks.load(Ordering::SeqCst)
	}

	pub fn fractional_position(&self) -> f64 {
		f64::from_bits(self.fractional_position.load(Ordering::SeqCst))
	}

	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum State {
	NotStarted,
	Started {
		ticks: u64,
		fractional_position: f64,
	},
}

#[derive(Clone)]
pub(crate) struct Clock {
	shared: Arc<ClockShared>,
	ticking: bool,
	speed: Parameter<ClockSpeed>,
	state: State,
}

impl Clock {
	pub(crate) fn new(speed: Value<ClockSpeed>) -> Self {
		Self {
			shared: Arc::new(ClockShared::new()),
			ticking: false,
			speed: Parameter::new(speed, ClockSpeed::TicksPerMinute(120.0)),
			state: State::NotStarted,
		}
	}

	pub(crate) fn shared(&self) -> Arc<ClockShared> {
		self.shared.clone()
	}

	pub(crate) fn set_speed(&mut self, speed: Value<ClockSpeed>, tween: Tween) {
		self.speed.set(speed, tween);
	}

	pub(crate) fn start(&mut self) {
		self.ticking = true;
		self.shared.ticking.store(true, Ordering::SeqCst);
	}

	pub(crate) fn pause(&mut self) {
		self.ticking = false;
		self.shared.ticking.store(false, Ordering::SeqCst);
	}

	pub(crate) fn stop(&mut self) {
		self.pause();
		self.state = State::NotStarted;
		self.shared.ticks.store(0, Ordering::SeqCst);
	}

	pub(crate) fn ticking(&self) -> bool {
		self.ticking
	}

	pub(crate) fn ticks(&self) -> u64 {
		match self.state {
			State::NotStarted => 0,
			State::Started { ticks, .. } => ticks,
		}
	}

	pub(crate) fn fractional_position(&self) -> f64 {
		match self.state {
			State::NotStarted => 0.0,
			State::Started {
				fractional_position,
				..
			} => fractional_position,
		}
	}

	pub(crate) fn on_start_processing(&mut self) {
		self.shared.ticks.store(self.ticks(), Ordering::SeqCst);
		self.shared
			.fractional_position
			.store(self.fractional_position().to_bits(), Ordering::SeqCst);
	}

	/// Updates the [`Clock`].
	///
	/// If the tick count changes this update, returns `Some(tick_number)`.
	/// Otherwise, returns `None`.
	pub(crate) fn update(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Option<u64> {
		self.speed
			.update(dt, clock_info_provider, modulator_value_provider);
		if !self.ticking {
			return None;
		}
		let mut new_tick_count = None;
		if self.state == State::NotStarted {
			self.state = State::Started {
				ticks: 0,
				fractional_position: 0.0,
			};
			new_tick_count = Some(0);
		}
		if let State::Started {
			ticks,
			fractional_position: tick_timer,
		} = &mut self.state
		{
			*tick_timer += self.speed.value().as_ticks_per_second() * dt;
			while *tick_timer >= 1.0 {
				*tick_timer -= 1.0;
				*ticks += 1;
				new_tick_count = Some(*ticks);
			}
		} else {
			panic!("clock state should be Started by now");
		}
		new_tick_count
	}
}
