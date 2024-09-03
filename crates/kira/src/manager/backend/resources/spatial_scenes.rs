use crate::{
	clock::clock_info::ClockInfoProvider,
	modulator::value_provider::ModulatorValueProvider,
	spatial::scene::{SpatialScene, SpatialSceneId},
};

use super::{ResourceController, ResourceStorage};

pub(crate) struct SpatialScenes(ResourceStorage<SpatialScene>);

impl SpatialScenes {
	#[must_use]
	pub fn new(capacity: u16) -> (Self, ResourceController<SpatialScene>) {
		let (storage, controller) = ResourceStorage::new(capacity);
		(Self(storage), controller)
	}

	#[must_use]
	pub fn get(&self, id: SpatialSceneId) -> Option<&SpatialScene> {
		self.0.get(id.0)
	}

	pub fn on_start_processing(&mut self) {
		self.0
			.remove_and_add(|scene| scene.shared().is_marked_for_removal());
		for (_, scene) in &mut self.0 {
			scene.on_start_processing();
		}
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) {
		for (_, scene) in &mut self.0 {
			scene.process(dt, clock_info_provider, modulator_value_provider);
		}
	}
}
