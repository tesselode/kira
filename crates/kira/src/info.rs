/*!
 * Types for providing info about resources to trait implementors.
 *
 * You'll only need this if you're implementing one of Kira's traits,
 * like [`Sound`](crate::sound::Sound) or [`Effect`](crate::effect::Effect).
 */

use glam::{Quat, Vec3};

use crate::{
	arena::Arena,
	clock::{Clock, ClockId, ClockTime, State as ClockState},
	listener::{Listener, ListenerId},
	modulator::{Modulator, ModulatorId},
};

/// Provides info about resources on the audio thread.
///
/// You'll only need this if you're implementing one of Kira's traits,
/// like [`Sound`](crate::sound::Sound) or [`Effect`](crate::effect::Effect).
pub struct Info<'a> {
	kind: InfoKind<'a>,
	spatial_track_info: Option<SpatialTrackInfo>,
}

impl<'a> Info<'a> {
	pub(crate) fn new(
		clocks: &'a Arena<Clock>,
		modulators: &'a Arena<Box<dyn Modulator>>,
		listeners: &'a Arena<Listener>,
		spatial_track_info: Option<SpatialTrackInfo>,
	) -> Self {
		Self {
			kind: InfoKind::Real {
				clocks,
				modulators,
				listeners,
			},
			spatial_track_info,
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

	/// Gets information about the listener linked to the current spatial track
	/// if there is one.
	#[must_use]
	pub fn listener_info(&self) -> Option<ListenerInfo> {
		self.spatial_track_info.and_then(|spatial_track_info| {
			let listener_id = spatial_track_info.listener_id;
			match &self.kind {
				InfoKind::Real { listeners, .. } => {
					listeners.get(listener_id.0).map(|listener| ListenerInfo {
						position: listener.position.value().into(),
						orientation: listener.orientation.value().into(),
						previous_position: listener.position.previous_value().into(),
						previous_orientation: listener.orientation.previous_value().into(),
					})
				}
				InfoKind::Mock { listener_info, .. } => listener_info.get(listener_id.0).copied(),
			}
		})
	}

	/// If this is called from an effect on a spatial track, returns the distance
	/// of the spatial track's from the spatial track. Otherwise, returns `None`.
	pub fn listener_distance(&self) -> Option<f32> {
		self.spatial_track_info.zip(self.listener_info()).map(
			|(spatial_track_info, listener_info)| {
				Vec3::from(listener_info.position).distance(spatial_track_info.position)
			},
		)
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

/// Information about a listener.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListenerInfo {
	/// The position of the listener.
	pub position: mint::Vector3<f32>,
	/// The rotation of the listener.
	pub orientation: mint::Quaternion<f32>,
	/// The position of the listener prior to the last update.
	pub previous_position: mint::Vector3<f32>,
	/// The rotation of the listener prior to the last update.
	pub previous_orientation: mint::Quaternion<f32>,
}

impl ListenerInfo {
	/// Returns the interpolated position between the previous and current
	/// position of the listener.
	pub fn interpolated_position(self, amount: f32) -> mint::Vector3<f32> {
		let position: Vec3 = self.position.into();
		let previous_position: Vec3 = self.previous_position.into();
		previous_position.lerp(position, amount).into()
	}

	/// Returns the interpolated orientation between the previous and current
	/// orientation of the listener.
	pub fn interpolated_orientation(self, amount: f32) -> mint::Quaternion<f32> {
		let orientation: Quat = self.orientation.into();
		let previous_orientation: Quat = self.previous_orientation.into();
		previous_orientation.lerp(orientation, amount).into()
	}
}

/// Generates a fake `Info` with arbitrary data. Useful for writing unit tests.
pub struct MockInfoBuilder {
	clock_info: Arena<ClockInfo>,
	modulator_values: Arena<f64>,
	listener_info: Arena<ListenerInfo>,
	spatial_track_info: Option<SpatialTrackInfo>,
}

impl MockInfoBuilder {
	/// Creates a new `MockInfoBuilder`.
	pub fn new() -> Self {
		Self {
			clock_info: Arena::new(100),
			modulator_values: Arena::new(100),
			listener_info: Arena::new(100),
			spatial_track_info: None,
		}
	}

	/// Adds a fake clock with the given ticking state and time. Returns a fake
	/// `ClockId`.
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

	/// Adds a fake modulator outputting the given value. Returns a fake `ModulatorId`.
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

	/// Adds a fake listener at the given position and orientation. Returns a fake `ListenerId`.
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

	/// Consumes the `MockInfoProvider` and returns a fake `Info`.
	pub fn build(self) -> Info<'static> {
		Info {
			kind: InfoKind::Mock {
				clock_info: self.clock_info,
				modulator_values: self.modulator_values,
				listener_info: self.listener_info,
			},
			spatial_track_info: self.spatial_track_info,
		}
	}
}

impl Default for MockInfoBuilder {
	fn default() -> Self {
		Self::new()
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SpatialTrackInfo {
	pub position: Vec3,
	pub listener_id: ListenerId,
}
