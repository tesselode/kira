use glam::Vec3;

use crate::{
	arena::Arena,
	clock::{Clock, ClockId, ClockTime, State as ClockState},
	listener::{Listener, ListenerId},
	modulator::{Modulator, ModulatorId},
};

pub struct Info<'a> {
	kind: InfoKind<'a>,
	spatial_track_position: Option<Vec3>,
}

impl<'a> Info<'a> {
	pub(crate) fn new(
		clocks: &'a Arena<Clock>,
		modulators: &'a Arena<Box<dyn Modulator>>,
		listeners: &'a Arena<Listener>,
		spatial_track_position: Option<Vec3>,
	) -> Self {
		Self {
			kind: InfoKind::Real {
				clocks,
				modulators,
				listeners,
			},
			spatial_track_position,
		}
	}

	/// Gets information about the clock with the given ID if it
	/// exists, returns `None` otherwise.
	#[must_use]
	pub fn clock_info(&self, id: ClockId) -> Option<ClockInfo> {
		match &self.kind {
			InfoKind::Real { clocks, .. } => clocks.get(id.0).map(|clock| ClockInfo {
				ticking: clock.ticking(),
				time: ClockTime {
					clock: id,
					ticks: match clock.state() {
						ClockState::NotStarted => 0,
						ClockState::Started { ticks, .. } => ticks,
					},
					fraction: match clock.state() {
						ClockState::NotStarted => 0.0,
						ClockState::Started {
							fractional_position,
							..
						} => fractional_position,
					},
				},
			}),
			InfoKind::Mock { clock_info, .. } => clock_info.get(id.0).copied(),
		}
	}

	/// Returns whether something with the given start time should
	/// start now, later, or never given the current state of the clocks.
	#[must_use]
	pub fn when_to_start(&self, time: ClockTime) -> WhenToStart {
		if let Some(clock_info) = self.clock_info(time.clock) {
			if clock_info.ticking && clock_info.time >= time {
				WhenToStart::Now
			} else {
				WhenToStart::Later
			}
		} else {
			WhenToStart::Never
		}
	}

	/// Gets the value of the modulator with the given ID if it
	/// exists, returns `None` otherwise.
	#[must_use]
	pub fn modulator_value(&self, id: ModulatorId) -> Option<f64> {
		match &self.kind {
			InfoKind::Real { modulators, .. } => {
				modulators.get(id.0).map(|modulator| modulator.value())
			}
			InfoKind::Mock {
				modulator_values, ..
			} => modulator_values.get(id.0).copied(),
		}
	}

	/// Gets information about the listener with the given ID if it
	/// exists, returns `None` otherwise.
	#[must_use]
	pub fn listener_info(&self, id: ListenerId) -> Option<ListenerInfo> {
		match &self.kind {
			InfoKind::Real { listeners, .. } => listeners.get(id.0).map(|listener| ListenerInfo {
				position: listener.position.value().into(),
				orientation: listener.orientation.value().into(),
			}),
			InfoKind::Mock { listener_info, .. } => listener_info.get(id.0).copied(),
		}
	}

	pub fn listener_distance(&self, id: ListenerId) -> Option<f32> {
		self.spatial_track_position.zip(self.listener_info(id)).map(
			|(spatial_track_position, listener_info)| {
				Vec3::from(listener_info.position).distance(spatial_track_position)
			},
		)
	}

	pub fn spatial_track_position(&self) -> Option<Vec3> {
		self.spatial_track_position
	}
}

/// Information about the current state of a [clock](super::clock).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClockInfo {
	/// Whether the clock is currently running.
	pub ticking: bool,
	/// The current time of the clock.
	pub time: ClockTime,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListenerInfo {
	pub position: mint::Vector3<f32>,
	pub orientation: mint::Quaternion<f32>,
}

pub struct MockInfoBuilder {
	clock_info: Arena<ClockInfo>,
	modulator_values: Arena<f64>,
	listener_info: Arena<ListenerInfo>,
	spatial_track_position: Option<Vec3>,
}

impl MockInfoBuilder {
	pub fn new(spatial_track_position: Option<Vec3>) -> Self {
		Self {
			clock_info: Arena::new(100),
			modulator_values: Arena::new(100),
			listener_info: Arena::new(100),
			spatial_track_position,
		}
	}

	pub fn add_clock(&mut self, ticking: bool, ticks: u64, fraction: f64) -> ClockId {
		let id = ClockId(
			self.clock_info
				.controller()
				.try_reserve()
				.expect("clock info arena is full"),
		);
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
		id
	}

	pub fn add_modulator(&mut self, value: f64) -> ModulatorId {
		let id = ModulatorId(
			self.modulator_values
				.controller()
				.try_reserve()
				.expect("modulator info arena is full"),
		);
		self.modulator_values.insert_with_key(id.0, value).unwrap();
		id
	}

	pub fn add_listener(&mut self, listener_info: ListenerInfo) -> ListenerId {
		let id = ListenerId(
			self.listener_info
				.controller()
				.try_reserve()
				.expect("listener info arena is full"),
		);
		self.listener_info
			.insert_with_key(id.0, listener_info)
			.unwrap();
		id
	}

	pub fn build(self) -> Info<'static> {
		Info {
			kind: InfoKind::Mock {
				clock_info: self.clock_info,
				modulator_values: self.modulator_values,
				listener_info: self.listener_info,
			},
			spatial_track_position: self.spatial_track_position,
		}
	}
}

enum InfoKind<'a> {
	Real {
		clocks: &'a Arena<Clock>,
		modulators: &'a Arena<Box<dyn Modulator>>,
		listeners: &'a Arena<Listener>,
	},
	Mock {
		clock_info: Arena<ClockInfo>,
		modulator_values: Arena<f64>,
		listener_info: Arena<ListenerInfo>,
	},
}
