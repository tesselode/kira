use crate::{
	clock::clock_info::ClockInfoProvider, modulator::value_provider::ModulatorValueProvider,
	sound::Sound, OutputDestination,
};

use super::{mixer::Mixer, spatial_scenes::SpatialScenes, ResourceController, ResourceStorage};

pub(crate) struct Sounds(ResourceStorage<Box<dyn Sound>>);

impl Sounds {
	#[must_use]
	pub fn new(capacity: u16) -> (Self, ResourceController<Box<dyn Sound>>) {
		let (storage, controller) = ResourceStorage::new(capacity);
		(Self(storage), controller)
	}

	pub fn on_start_processing(&mut self) {
		self.0.remove_and_add(|sound| sound.finished());
		for (_, sound) in &mut self.0 {
			sound.on_start_processing();
		}
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
		mixer: &mut Mixer,
		scenes: &mut SpatialScenes,
	) {
		for (_, sound) in &mut self.0 {
			match sound.output_destination() {
				OutputDestination::Track(track_id) => {
					if let Some(track) = mixer.track_mut(track_id) {
						track.add_input(sound.process(
							dt,
							clock_info_provider,
							modulator_value_provider,
						));
					}
				}
				OutputDestination::Emitter(emitter_id) => {
					if let Some(scene) = scenes.get_mut(emitter_id.scene_id) {
						if let Some(emitter) = scene.emitter_mut(emitter_id) {
							emitter.add_input(sound.process(
								dt,
								clock_info_provider,
								modulator_value_provider,
							));
						}
					}
				}
			}
		}
	}
}
