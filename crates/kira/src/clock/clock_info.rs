//! Contains types for reporting information about clocks.s
//!
//! You'll only need these types if you're creating implementations
//! of the [`Sound`](crate::sound::Sound) or
//! [`Effect`](crate::track::effect::Effect) traits. If you want
//! to access information about clocks from gameplay code, use
//! a [`ClockHandle`](crate::clock::ClockHandle).

use atomic_arena::{error::ArenaFull, Arena};

use crate::{manager::backend::resources::clocks::Clocks, StartTime};

use super::{ClockId, ClockTime};

/// Information about the current state of a [clock](super).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClockInfo {
	/// Whether the clock is currently running.
	pub ticking: bool,
	/// The number of times the clock has ticked.
	pub ticks: u64,
	/// The time between ticks (from `0.0`-`1.0`).
	pub fractional_position: f64,
}

/// Provides information about any clock that currently exists.
pub struct ClockInfoProvider<'a> {
	kind: ClockInfoProviderKind<'a>,
}

impl<'a> ClockInfoProvider<'a> {
	pub(crate) fn new(clocks: &'a Clocks) -> Self {
		Self {
			kind: ClockInfoProviderKind::Normal { clocks },
		}
	}

	/// Gets information about the clock with the given ID if it
	/// exists, returns `None` otherwise.
	pub fn get(&self, id: ClockId) -> Option<ClockInfo> {
		match &self.kind {
			ClockInfoProviderKind::Normal { clocks } => clocks.get(id).map(|clock| ClockInfo {
				ticking: clock.shared.ticking(),
				ticks: clock.shared.ticks(),
				fractional_position: clock.shared.fractional_position(),
			}),
			ClockInfoProviderKind::Mock { clock_info } => clock_info.get(id.0).copied(),
		}
	}

	/// Returns `true` if something with the given start time should
	/// start now given the current state of the clocks.
	pub fn should_start(&self, start_time: StartTime) -> bool {
		if let StartTime::ClockTime(ClockTime { clock, ticks }) = start_time {
			if let Some(clock_info) = self.get(clock) {
				clock_info.ticking && clock_info.ticks >= ticks
			} else {
				false
			}
		} else {
			true
		}
	}
}

enum ClockInfoProviderKind<'a> {
	Normal { clocks: &'a Clocks },
	Mock { clock_info: Arena<ClockInfo> },
}

/// Builds a `ClockInfoProvider` that provides fake clock info.
///
/// This is mainly useful for writing tests for implementations
/// of the [`Sound`](crate::sound::Sound) and
/// [`Effect`](crate::track::effect::Effect) traits.
pub struct MockClockInfoProviderBuilder {
	clock_info: Arena<ClockInfo>,
}

impl MockClockInfoProviderBuilder {
	/// Creates a new [`MockClockInfoProviderBuilder`] with room for
	/// the specified number of clocks.
	pub fn new(capacity: usize) -> Self {
		Self {
			clock_info: Arena::new(capacity),
		}
	}

	/// Adds a new fake clock to the builder and returns the corresponding
	/// [`ClockId`].
	pub fn add(&mut self, info: ClockInfo) -> Result<ClockId, ArenaFull> {
		Ok(ClockId(self.clock_info.insert(info)?))
	}

	/// Consumes the builder and returns a [`ClockInfoProvider`].
	pub fn build(self) -> ClockInfoProvider<'static> {
		ClockInfoProvider {
			kind: ClockInfoProviderKind::Mock {
				clock_info: self.clock_info,
			},
		}
	}
}
