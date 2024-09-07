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
clock.start();
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
manager.play(
	StaticSoundData::from_file("sound.ogg")?
		.start_time(StartTime::ClockTime(ClockTime {
			clock: clock.id(),
			ticks: 4,
			fraction: 0.0,
		}))
)?;
clock.start();
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
manager.play(
	StaticSoundData::from_file("sound.ogg")?
		.start_time(ClockTime {
			clock: clock.id(),
			ticks: 4,
			fraction: 0.0,
		})
)?;
# clock.start();
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
manager.play(
	StaticSoundData::from_file("sound.ogg")?
		.start_time(clock.time() + 4)
)?;
# clock.start();
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
let mut sound = manager.play(StaticSoundData::from_file("sound.ogg")?)?;
sound.set_playback_rate(
	0.5,
	Tween {
		start_time: StartTime::ClockTime(clock.time() + 3),
		duration: Duration::from_secs(2),
		..Default::default()
	},
);
clock.start();
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

use crate::{arena::Key, listener::ListenerInfoProvider};

use crate::{
	command::read_commands_into_parameters,
	command::ValueChangeCommand,
	command_writers_and_readers,
	modulator::value_provider::ModulatorValueProvider,
	tween::{Parameter, Value},
};

use self::clock_info::ClockInfoProvider;

/// A unique identifier for a clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockId(pub(crate) Key);

#[derive(Debug)]
pub(crate) struct ClockShared {
	ticking: AtomicBool,
	ticks: AtomicU64,
	fractional_position: AtomicU64,
	removed: AtomicBool,
}

impl ClockShared {
	#[must_use]
	pub fn new() -> Self {
		Self {
			ticking: AtomicBool::new(false),
			ticks: AtomicU64::new(0),
			fractional_position: AtomicU64::new(0.0f64.to_bits()),
			removed: AtomicBool::new(false),
		}
	}

	#[must_use]
	pub fn ticking(&self) -> bool {
		self.ticking.load(Ordering::SeqCst)
	}

	#[must_use]
	pub fn ticks(&self) -> u64 {
		self.ticks.load(Ordering::SeqCst)
	}

	#[must_use]
	pub fn fractional_position(&self) -> f64 {
		f64::from_bits(self.fractional_position.load(Ordering::SeqCst))
	}

	#[must_use]
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

pub(crate) struct Clock {
	command_readers: CommandReaders,
	shared: Arc<ClockShared>,
	ticking: bool,
	speed: Parameter<ClockSpeed>,
	state: State,
}

impl Clock {
	#[must_use]
	pub(crate) fn new(speed: Value<ClockSpeed>, id: ClockId) -> (Self, ClockHandle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		let shared = Arc::new(ClockShared::new());
		(
			Self {
				command_readers,
				shared: shared.clone(),
				ticking: false,
				speed: Parameter::new(speed, ClockSpeed::TicksPerMinute(120.0)),
				state: State::NotStarted,
			},
			ClockHandle {
				id,
				shared,
				command_writers,
			},
		)
	}

	#[must_use]
	pub(crate) fn without_handle(speed: Value<ClockSpeed>) -> Self {
		let (_, command_readers) = command_writers_and_readers();
		Self {
			command_readers,
			shared: Arc::new(ClockShared::new()),
			ticking: false,
			speed: Parameter::new(speed, ClockSpeed::TicksPerMinute(120.0)),
			state: State::NotStarted,
		}
	}

	#[must_use]
	pub(crate) fn shared(&self) -> Arc<ClockShared> {
		self.shared.clone()
	}

	#[must_use]
	pub(crate) fn state(&self) -> State {
		self.state
	}

	#[must_use]
	pub(crate) fn ticking(&self) -> bool {
		self.ticking
	}

	pub(crate) fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, speed);
		if let Some(ticking) = self.command_readers.set_ticking.read() {
			self.set_ticking(ticking);
		}
		if self.command_readers.reset.read().is_some() {
			self.reset();
		}
		self.update_shared();
	}

	fn set_ticking(&mut self, ticking: bool) {
		self.ticking = ticking;
		self.shared.ticking.store(ticking, Ordering::SeqCst);
	}

	fn reset(&mut self) {
		self.state = State::NotStarted;
		self.shared.ticks.store(0, Ordering::SeqCst);
	}

	fn update_shared(&mut self) {
		let (ticks, fractional_position) = match &self.state {
			State::NotStarted => (0, 0.0),
			State::Started {
				ticks,
				fractional_position,
			} => (*ticks, *fractional_position),
		};
		self.shared.ticks.store(ticks, Ordering::SeqCst);
		self.shared
			.fractional_position
			.store(fractional_position.to_bits(), Ordering::SeqCst);
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
		listener_info_provider: &ListenerInfoProvider,
	) -> Option<u64> {
		self.speed.update(
			dt,
			clock_info_provider,
			modulator_value_provider,
			listener_info_provider,
		);
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

impl Default for Clock {
	fn default() -> Self {
		Self::without_handle(Value::Fixed(ClockSpeed::TicksPerSecond(0.0)))
	}
}

command_writers_and_readers! {
	set_speed: ValueChangeCommand<ClockSpeed>,
	set_ticking: bool,
	reset: (),
}
