use crate::tween::Tweenable;

/// The rate that a [clock](crate::clock) ticks at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClockSpeed {
	/// The clock ticks every x seconds.
	SecondsPerTick(f64),
	/// The clock ticks x times per second.
	TicksPerSecond(f64),
	/// The clock ticks x times per minute.
	TicksPerMinute(f64),
}

impl ClockSpeed {
	/// Returns the [`ClockSpeed`] as a number of seconds between each tick.
	pub fn as_seconds_per_tick(&self) -> f64 {
		match self {
			ClockSpeed::SecondsPerTick(seconds_per_tick) => *seconds_per_tick,
			ClockSpeed::TicksPerSecond(ticks_per_second) => 1.0 / *ticks_per_second,
			ClockSpeed::TicksPerMinute(ticks_per_minute) => 60.0 / *ticks_per_minute,
		}
	}

	/// Returns the [`ClockSpeed`] as a number of ticks per second.
	pub fn as_ticks_per_second(&self) -> f64 {
		match self {
			ClockSpeed::SecondsPerTick(seconds_per_tick) => 1.0 / *seconds_per_tick,
			ClockSpeed::TicksPerSecond(ticks_per_second) => *ticks_per_second,
			ClockSpeed::TicksPerMinute(ticks_per_minute) => *ticks_per_minute / 60.0,
		}
	}

	/// Returns the [`ClockSpeed`] as a number of ticks per minute.
	pub fn as_ticks_per_minute(&self) -> f64 {
		match self {
			ClockSpeed::SecondsPerTick(seconds_per_tick) => 60.0 / *seconds_per_tick,
			ClockSpeed::TicksPerSecond(ticks_per_second) => *ticks_per_second * 60.0,
			ClockSpeed::TicksPerMinute(ticks_per_minute) => *ticks_per_minute,
		}
	}
}

impl Tweenable for ClockSpeed {
	fn interpolate(a: Self, b: Self, amount: f64) -> Self {
		match b {
			ClockSpeed::SecondsPerTick(b) => ClockSpeed::SecondsPerTick(Tweenable::interpolate(
				a.as_seconds_per_tick(),
				b,
				amount,
			)),
			ClockSpeed::TicksPerSecond(b) => ClockSpeed::TicksPerSecond(Tweenable::interpolate(
				a.as_ticks_per_second(),
				b,
				amount,
			)),
			ClockSpeed::TicksPerMinute(b) => ClockSpeed::TicksPerMinute(Tweenable::interpolate(
				a.as_ticks_per_minute(),
				b,
				amount,
			)),
		}
	}
}

#[cfg(test)]
#[test]
#[allow(clippy::float_cmp)]
fn test() {
	const SECONDS_PER_TICK: f64 = 0.5;
	const TICKS_PER_SECOND: f64 = 2.0;
	const TICKS_PER_MINUTE: f64 = 120.0;

	assert_eq!(
		ClockSpeed::SecondsPerTick(SECONDS_PER_TICK).as_seconds_per_tick(),
		SECONDS_PER_TICK
	);
	assert_eq!(
		ClockSpeed::SecondsPerTick(SECONDS_PER_TICK).as_ticks_per_second(),
		TICKS_PER_SECOND
	);
	assert_eq!(
		ClockSpeed::SecondsPerTick(SECONDS_PER_TICK).as_ticks_per_minute(),
		TICKS_PER_MINUTE
	);

	assert_eq!(
		ClockSpeed::TicksPerSecond(TICKS_PER_SECOND).as_seconds_per_tick(),
		SECONDS_PER_TICK
	);
	assert_eq!(
		ClockSpeed::TicksPerSecond(TICKS_PER_SECOND).as_ticks_per_second(),
		TICKS_PER_SECOND
	);
	assert_eq!(
		ClockSpeed::TicksPerSecond(TICKS_PER_SECOND).as_ticks_per_minute(),
		TICKS_PER_MINUTE
	);

	assert_eq!(
		ClockSpeed::TicksPerMinute(TICKS_PER_MINUTE).as_seconds_per_tick(),
		SECONDS_PER_TICK
	);
	assert_eq!(
		ClockSpeed::TicksPerMinute(TICKS_PER_MINUTE).as_ticks_per_second(),
		TICKS_PER_SECOND
	);
	assert_eq!(
		ClockSpeed::TicksPerMinute(TICKS_PER_MINUTE).as_ticks_per_minute(),
		TICKS_PER_MINUTE
	);
}
