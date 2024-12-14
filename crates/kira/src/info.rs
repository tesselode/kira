/*!
 * Types for providing info about resources to trait implementors.
 *
 * You'll only need this if you're implementing one of Kira's traits,
 * like [`Sound`](crate::sound::Sound) or [`Effect`](crate::effect::Effect).
 */

mod mock_info_builder;
mod single_frame_info;

pub use mock_info_builder::*;
pub use single_frame_info::*;

// use glam::Vec3;

use crate::{
	arena::Arena,
	clock::{ClockId, ClockInfo},
	// listener::{Listener, ListenerId},
	modulator::ModulatorId,
	resources::{
		clocks::buffered_clock::BufferedClock, modulators::buffered_modulator::BufferedModulator,
	},
};

/// Provides info about resources on the audio thread.
///
/// You'll only need this if you're implementing one of Kira's traits,
/// like [`Sound`](crate::sound::Sound) or [`Effect`](crate::effect::Effect).
pub struct Info<'a> {
	kind: InfoKind<'a>,
	// spatial_track_info: Option<SpatialTrackInfo>,
}

impl<'a> Info<'a> {
	pub(crate) fn new(
		clocks: &'a Arena<BufferedClock>,
		modulators: &'a Arena<BufferedModulator>,
		// listeners: &'a Arena<Listener>,
		// spatial_track_info: Option<SpatialTrackInfo>,
	) -> Self {
		Self {
			kind: InfoKind::Real {
				clocks,
				modulators,
				// listeners,
			},
			// spatial_track_info,
		}
	}

	pub fn for_single_frame(&self, frame_index: usize) -> SingleFrameInfo {
		SingleFrameInfo {
			info: self,
			info_frame: InfoFrame::Specified(frame_index),
		}
	}

	pub(crate) fn latest(&self) -> SingleFrameInfo {
		SingleFrameInfo {
			info: self,
			info_frame: InfoFrame::Latest,
		}
	}

	/// Gets information about the clock with the given ID if it
	/// exists, returns `None` otherwise.
	#[must_use]
	pub fn clock_info(&self, id: ClockId) -> Option<&[ClockInfo]> {
		match &self.kind {
			InfoKind::Real { clocks, .. } => clocks.get(id.0).map(|clock| clock.info_buffer()),
			InfoKind::Mock { clock_info, .. } => clock_info.get(id.0).map(|info| info.as_slice()),
		}
	}

	/// Gets the value of the modulator with the given ID if it
	/// exists, returns `None` otherwise.
	#[must_use]
	pub fn modulator_values(&self, id: ModulatorId) -> Option<&[f64]> {
		match &self.kind {
			InfoKind::Real { modulators, .. } => modulators
				.get(id.0)
				.map(|modulator| modulator.value_buffer()),
			InfoKind::Mock {
				modulator_values, ..
			} => modulator_values.get(id.0).map(|values| values.as_slice()),
		}
	}

	fn latest_clock_info(&self, id: ClockId) -> Option<ClockInfo> {
		match &self.kind {
			InfoKind::Real { clocks, .. } => clocks.get(id.0).map(|clock| clock.info()),
			InfoKind::Mock { clock_info, .. } => {
				clock_info.get(id.0).and_then(|info| info.last().copied())
			}
		}
	}

	fn latest_modulator_value(&self, id: ModulatorId) -> Option<f64> {
		match &self.kind {
			InfoKind::Real { modulators, .. } => {
				modulators.get(id.0).map(|modulator| modulator.value())
			}
			InfoKind::Mock {
				modulator_values, ..
			} => modulator_values
				.get(id.0)
				.and_then(|values| values.last().copied()),
		}
	}

	/* /// Gets information about the listener linked to the current spatial track
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
					})
				}
				InfoKind::Mock { listener_info, .. } => listener_info.get(listener_id.0).copied(),
			}
		})
	} */

	/* /// If this is called from an effect on a spatial track, returns the distance
	/// of the spatial track's from the spatial track. Otherwise, returns `None`.
	pub fn listener_distance(&self) -> Option<f32> {
		self.spatial_track_info.zip(self.listener_info()).map(
			|(spatial_track_info, listener_info)| {
				Vec3::from(listener_info.position).distance(spatial_track_info.position)
			},
		)
	} */
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

/* /// Information about a listener.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListenerInfo {
	/// The position of the listener.
	pub position: mint::Vector3<f32>,
	/// The rotation of the listener.
	pub orientation: mint::Quaternion<f32>,
} */

enum InfoKind<'a> {
	Real {
		clocks: &'a Arena<BufferedClock>,
		modulators: &'a Arena<BufferedModulator>,
		// listeners: &'a Arena<Listener>,
	},
	Mock {
		clock_info: Arena<Vec<ClockInfo>>,
		modulator_values: Arena<Vec<f64>>,
		// listener_info: Arena<ListenerInfo>,
	},
}

/* #[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SpatialTrackInfo {
	pub position: Vec3,
	pub listener_id: ListenerId,
} */
