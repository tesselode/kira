use crate::{
	arena::Arena,
	clock::{ClockId, ClockInfo},
	modulator::ModulatorId,
};

use super::{Info, InfoKind};

/// Generates a fake `Info` with arbitrary data. Useful for writing unit tests.
pub struct MockInfoBuilder {
	clock_info: Arena<Vec<ClockInfo>>,
	modulator_values: Arena<Vec<f64>>,
	// listener_info: Arena<ListenerInfo>,
	// spatial_track_info: Option<SpatialTrackInfo>,
}

impl MockInfoBuilder {
	/// Creates a new `MockInfoBuilder`.
	pub fn new() -> Self {
		Self {
			clock_info: Arena::new(100),
			modulator_values: Arena::new(100),
			// listener_info: Arena::new(100),
			// spatial_track_info: None,
		}
	}

	/// Adds a fake clock with the given ticking state and time. Returns a fake
	/// `ClockId`.
	pub fn add_clock(&mut self, clock_info: Vec<ClockInfo>) -> ClockId {
		let id = ClockId(
			self.clock_info
				.controller()
				.try_reserve()
				.expect("clock info arena is full"),
		);
		self.clock_info.insert_with_key(id.0, clock_info).unwrap();
		id
	}

	/// Adds a fake modulator outputting the given value. Returns a fake `ModulatorId`.
	pub fn add_modulator(&mut self, values: Vec<f64>) -> ModulatorId {
		let id = ModulatorId(
			self.modulator_values
				.controller()
				.try_reserve()
				.expect("modulator info arena is full"),
		);
		self.modulator_values.insert_with_key(id.0, values).unwrap();
		id
	}

	/* /// Adds a fake listener at the given position and orientation. Returns a fake `ListenerId`.
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
	} */

	/// Consumes the `MockInfoProvider` and returns a fake `Info`.
	pub fn build(self) -> Info<'static> {
		Info {
			kind: InfoKind::Mock {
				clock_info: self.clock_info,
				modulator_values: self.modulator_values,
				// listener_info: self.listener_info,
			},
			// spatial_track_info: self.spatial_track_info,
		}
	}
}

impl Default for MockInfoBuilder {
	fn default() -> Self {
		Self::new()
	}
}
