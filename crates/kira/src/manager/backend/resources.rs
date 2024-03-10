pub(crate) mod mixer;
pub(crate) mod modulators;
pub(crate) mod sounds;
pub(crate) mod spatial_scenes;

use std::sync::Mutex;

use atomic_arena::Controller;
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};

use crate::{
	clock::{Clock, ClockId},
	manager::settings::Capacities,
	modulator::Modulator,
	resource::Clocks,
	sound::wrapper::SoundWrapper,
	spatial::scene::SpatialScene,
	track::{Track, TrackBuilder, TrackHandle},
};

use self::{mixer::Mixer, modulators::Modulators, sounds::Sounds, spatial_scenes::SpatialScenes};

pub(crate) struct NewResourceProducers {
	pub clock: HeapProducer<(ClockId, Clock)>,
}

pub(crate) struct UnusedResourceConsumers {
	pub sound: Mutex<HeapConsumer<SoundWrapper>>,
	pub sub_track: Mutex<HeapConsumer<Track>>,
	pub clock: Mutex<HeapConsumer<Clock>>,
	pub spatial_scene: Mutex<HeapConsumer<SpatialScene>>,
	pub modulator: Mutex<HeapConsumer<Box<dyn Modulator>>>,
}

pub(crate) struct Resources {
	pub sounds: Sounds,
	pub mixer: Mixer,
	pub clocks: Clocks,
	pub spatial_scenes: SpatialScenes,
	pub modulators: Modulators,
}

pub(crate) struct ResourceControllers {
	pub sound_controller: Controller,
	pub sub_track_controller: Controller,
	pub clock_controller: Controller,
	pub spatial_scene_controller: Controller,
	pub modulator_controller: Controller,
}

pub(crate) fn create_resources(
	capacities: Capacities,
	main_track_builder: TrackBuilder,
	sample_rate: u32,
) -> (
	Resources,
	ResourceControllers,
	NewResourceProducers,
	UnusedResourceConsumers,
	TrackHandle,
) {
	let (unused_sound_producer, unused_sound_consumer) =
		HeapRb::new(capacities.sound_capacity).split();
	let (unused_sub_track_producer, unused_sub_track_consumer) =
		HeapRb::new(capacities.sub_track_capacity).split();
	let (unused_spatial_scene_producer, unused_spatial_scene_consumer) =
		HeapRb::new(capacities.spatial_scene_capacity).split();
	let (unused_modulator_producer, unused_modulator_consumer) =
		HeapRb::new(capacities.modulator_capacity).split();
	let sounds = Sounds::new(capacities.sound_capacity, unused_sound_producer);
	let sound_controller = sounds.controller();
	let (mixer, main_track_handle) = Mixer::new(
		capacities.sub_track_capacity,
		unused_sub_track_producer,
		sample_rate,
		main_track_builder,
	);
	let sub_track_controller = mixer.sub_track_controller();
	let (clocks, clock_controller, new_clock_producer, unused_clock_consumer) =
		Clocks::new(capacities.clock_capacity);
	let spatial_scenes = SpatialScenes::new(
		capacities.spatial_scene_capacity,
		unused_spatial_scene_producer,
	);
	let spatial_scene_controller = spatial_scenes.controller();
	let modulators = Modulators::new(capacities.modulator_capacity, unused_modulator_producer);
	let modulator_controller = modulators.controller();
	(
		Resources {
			sounds,
			mixer,
			clocks,
			spatial_scenes,
			modulators,
		},
		ResourceControllers {
			sound_controller,
			sub_track_controller,
			clock_controller,
			spatial_scene_controller,
			modulator_controller,
		},
		NewResourceProducers {
			clock: new_clock_producer,
		},
		UnusedResourceConsumers {
			sound: Mutex::new(unused_sound_consumer),
			sub_track: Mutex::new(unused_sub_track_consumer),
			clock: Mutex::new(unused_clock_consumer),
			spatial_scene: Mutex::new(unused_spatial_scene_consumer),
			modulator: Mutex::new(unused_modulator_consumer),
		},
		main_track_handle,
	)
}
