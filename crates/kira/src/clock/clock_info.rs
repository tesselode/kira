//! Contains types for reporting information about clocks.
//!
//! You'll only need these types if you're creating implementations
//! of the [`Sound`](crate::sound::Sound),
//! [`Effect`](crate::track::effect::Effect), or
//! [`Modulator`](crate::modulator::Modulator) traits. If you want
//! to access information about clocks from gameplay code, use
//! a [`ClockHandle`](crate::clock::ClockHandle).

use atomic_arena::{error::ArenaFull, Arena};

use crate::manager::backend::resources::clocks::Clocks;

use super::{ClockId, ClockTime, State};

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
				ticking: clock.ticking(),
				ticks: match clock.state() {
					State::NotStarted => 0,
					State::Started { ticks, .. } => ticks,
				},
				fractional_position: match clock.state() {
					State::NotStarted => 0.0,
					State::Started {
						fractional_position,
						..
					} => fractional_position,
				},
			}),
			ClockInfoProviderKind::Mock { clock_info } => clock_info.get(id.0).copied(),
		}
	}

	/// Returns whether something with the given start time should
	/// start now, later, or never given the current state of the clocks.
	pub fn when_to_start(&self, ClockTime { clock, ticks }: ClockTime) -> WhenToStart {
		if let Some(clock_info) = self.get(clock) {
			if clock_info.ticking && clock_info.ticks >= ticks {
				WhenToStart::Now
			} else {
				WhenToStart::Later
			}
		} else {
			WhenToStart::Never
		}
	}
}

enum ClockInfoProviderKind<'a> {
	Normal { clocks: &'a Clocks },
	Mock { clock_info: Arena<ClockInfo> },
}

/// When something should start given the current state
/// of the clocks.
///
/// The "something" in question can be anything that
/// would start at a given [`StartTime`](crate::StartTime).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WhenToStart {
	/// The thing should start now.
	Now,
	/// The thing should start later because the appropriate
	/// clock isn't ticking or hasn't reached the target tick
	/// yet.
	Later,
	/// The thing will never start because the clock it depends
	/// on no longer exists.
	Never,
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
