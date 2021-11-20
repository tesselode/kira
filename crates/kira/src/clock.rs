/*!
Precise timing for audio events.

## Creating clocks

Clocks can be used to set the start times of sounds and
tweens. To create a clock, use [`AudioManager::add_clock`](crate::manager::AudioManager::add_clock):

```no_run
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
explicitly call [`ClockHandle::start`] when you want the clock
to start ticking.

## Starting sounds on clock ticks

Static sounds (and streaming sounds from the
[`kira-streaming`](https://crates.io/crates/kira-streaming) crate)
can be set to only start playing when a clock has ticked
a certain number of times. You can configure this using
[`StaticSoundSettings::start_time`](crate::sound::static_sound::StaticSoundSettings::start_time):

```no_run
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

As a shorthand, you can pass the [`ClockTime`] directly into
[`StaticSoundSettings::start_time`](crate::sound::static_sound::StaticSoundSettings::start_time):

```no_run
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

As an even shorter hand, you can use [`ClockHandle::time`] to get
the clock's current [`ClockTime`], and then add to it to get
a time in the future:

```no_run
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

```no_run
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
*/

mod clocks;
mod handle;
mod time;

pub use clocks::*;
pub use handle::*;
pub use time::*;

use std::sync::{
	atomic::{AtomicBool, AtomicU64, Ordering},
	Arc,
};

use atomic_arena::Key;

use crate::{
	parameter::Parameters,
	value::{CachedValue, Value},
};

/// A unique identifier for a [`Clock`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockId(pub(crate) Key);

pub(crate) struct ClockShared {
	ticking: AtomicBool,
	ticks: AtomicU64,
	removed: AtomicBool,
}

impl ClockShared {
	pub fn new() -> Self {
		Self {
			ticking: AtomicBool::new(false),
			ticks: AtomicU64::new(0),
			removed: AtomicBool::new(false),
		}
	}

	pub fn ticking(&self) -> bool {
		self.ticking.load(Ordering::SeqCst)
	}

	pub fn ticks(&self) -> u64 {
		self.ticks.load(Ordering::SeqCst)
	}

	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

/// A user-controllable timing source.
///
/// You will only need to interact with [`Clock`]s directly
/// if you're writing your own [`Sound`](crate::sound::Sound)s.
/// Otherwise, you'll be interacting with clocks using a
/// [`ClockHandle`].
pub struct Clock {
	shared: Arc<ClockShared>,
	ticking: bool,
	interval: CachedValue,
	ticks: u64,
	tick_timer: f64,
}

impl Clock {
	pub(crate) fn new(interval: Value) -> Self {
		Self {
			shared: Arc::new(ClockShared::new()),
			ticking: false,
			interval: CachedValue::new(0.0.., interval, 1.0),
			ticks: 0,
			tick_timer: 1.0,
		}
	}

	pub(crate) fn shared(&self) -> Arc<ClockShared> {
		self.shared.clone()
	}

	/// Returns `true` if the clock is currently running.
	pub fn ticking(&self) -> bool {
		self.ticking
	}

	/// Returns the number of times the clock has ticked.
	pub fn ticks(&self) -> u64 {
		self.ticks
	}

	pub(crate) fn set_interval(&mut self, interval: Value) {
		self.interval.set(interval);
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
		self.ticks = 0;
		self.shared.ticks.store(0, Ordering::SeqCst);
	}

	pub(crate) fn update(&mut self, dt: f64, parameters: &Parameters) {
		self.interval.update(parameters);
		if self.ticking {
			self.tick_timer -= dt / self.interval.get();
			while self.tick_timer <= 0.0 {
				self.tick_timer += 1.0;
				self.ticks += 1;
				self.shared.ticks.fetch_add(1, Ordering::SeqCst);
			}
		}
	}
}
