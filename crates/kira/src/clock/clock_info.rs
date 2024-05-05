//! Contains types for reporting information about clocks.
//!
//! You'll only need these types if you're creating implementations
//! of the [`Sound`](crate::sound::Sound),
//! [`Effect`](crate::track::effect::Effect), or
//! [`Modulator`](crate::modulator::Modulator) traits. If you want
//! to access information about clocks from gameplay code, use
//! a [`ClockHandle`](crate::clock::ClockHandle).

use crate::arena::{error::ArenaFull, Arena};

use super::{Clock, ClockId, ClockTime, State};

/// Information about the current state of a [clock](super).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClockInfo {
	/// Whether the clock is currently running.
	pub ticking: bool,
	/// The current time of the clock.
	pub time: ClockTime,
}

/// Provides information about any clock that currently exists.
pub struct ClockInfoProvider<'a> {
	kind: ClockInfoProviderKind<'a>,
}

impl<'a> ClockInfoProvider<'a> {
	#[must_use]
	pub(crate) fn new(clocks: &'a Arena<Clock>) -> Self {
		Self {
			kind: ClockInfoProviderKind::Normal { clocks },
		}
	}

	/// Gets information about the clock with the given ID if it
	/// exists, returns `None` otherwise.
	#[must_use]
	pub fn get(&self, id: ClockId) -> Option<ClockInfo> {
		match &self.kind {
			ClockInfoProviderKind::Normal { clocks } => clocks.get(id.0).map(|clock| ClockInfo {
				ticking: clock.ticking(),
				time: ClockTime {
					clock: id,
					ticks: match clock.state() {
						State::NotStarted => 0,
						State::Started { ticks, .. } => ticks,
					},
					fraction: match clock.state() {
						State::NotStarted => 0.0,
						State::Started {
							fractional_position,
							..
						} => fractional_position,
					},
				},
			}),
			ClockInfoProviderKind::Mock { clock_info } => clock_info.get(id.0).copied(),
		}
	}

	/// Returns whether something with the given start time should
	/// start now, later, or never given the current state of the clocks.
	#[must_use]
	pub fn when_to_start(&self, time: ClockTime) -> WhenToStart {
		if let Some(clock_info) = self.get(time.clock) {
			if clock_info.ticking && clock_info.time >= time {
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
	Normal { clocks: &'a Arena<Clock> },
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
	#[must_use]
	pub fn new(capacity: u16) -> Self {
		Self {
			clock_info: Arena::new(capacity),
		}
	}

	/// Adds a new fake clock to the builder and returns the corresponding
	/// [`ClockId`].
	pub fn add(&mut self, ticking: bool, ticks: u64, fraction: f64) -> Result<ClockId, ArenaFull> {
		let id = ClockId(self.clock_info.controller().try_reserve()?);
		self.clock_info
			.insert_with_key(
				id.0,
				ClockInfo {
					ticking,
					time: ClockTime {
						clock: id,
						ticks,
						fraction,
					},
				},
			)
			.unwrap();
		Ok(id)
	}

	/// Consumes the builder and returns a [`ClockInfoProvider`].
	#[must_use]
	pub fn build(self) -> ClockInfoProvider<'static> {
		ClockInfoProvider {
			kind: ClockInfoProviderKind::Mock {
				clock_info: self.clock_info,
			},
		}
	}
}
