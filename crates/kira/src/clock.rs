mod clock_speed;
mod handle;
mod shared;
mod time;
/* #[cfg(test)]
mod test; */

pub use clock_speed::*;
pub use handle::*;
use shared::ClockShared;
pub use time::*;

use std::sync::{atomic::Ordering, Arc};

use crate::{arena::Key, command_writers_and_readers};

/// A unique identifier for a clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockId(pub(crate) Key);

/// Information about the current state of a [clock](super::clock).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ClockInfo {
	/// Whether the clock is currently running.
	pub ticking: bool,
	/// The elapsed time in whole ticks.
	pub ticks: u64,
	/// The amount of time since the last tick as a fraction of a tick.
	///
	/// This will always be in the range of `0.0` (inclusive) to `1.0` (exclusive).
	pub fraction: f64,
}

pub(crate) struct Clock {
	command_readers: CommandReaders,
	shared: Arc<ClockShared>,
	ticking: bool,
	speed: ClockSpeed,
	state: State,
}

impl Clock {
	#[must_use]
	pub(crate) fn new(speed: ClockSpeed, id: ClockId) -> (Self, ClockHandle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		let shared = Arc::new(ClockShared::new());
		(
			Self {
				command_readers,
				shared: shared.clone(),
				ticking: false,
				speed,
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
	pub(crate) fn without_handle(speed: ClockSpeed) -> Self {
		let (_, command_readers) = command_writers_and_readers();
		Self {
			command_readers,
			shared: Arc::new(ClockShared::new()),
			ticking: false,
			speed,
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
		// read_commands_into_parameters!(self, speed);
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
	pub(crate) fn update(&mut self, dt: f64) {
		if !self.ticking {
			return;
		}
		if self.state == State::NotStarted {
			self.state = State::Started {
				ticks: 0,
				fractional_position: 0.0,
			};
		}
		if let State::Started {
			ticks,
			fractional_position: tick_timer,
		} = &mut self.state
		{
			*tick_timer += self.speed.as_ticks_per_second() * dt;
			while *tick_timer >= 1.0 {
				*tick_timer -= 1.0;
				*ticks += 1;
			}
		} else {
			panic!("clock state should be Started by now");
		}
	}
}

impl Default for Clock {
	fn default() -> Self {
		Self::without_handle(ClockSpeed::TicksPerSecond(0.0))
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

command_writers_and_readers! {
	// set_speed: ValueChangeCommand<ClockSpeed>,
	set_ticking: bool,
	reset: (),
}
