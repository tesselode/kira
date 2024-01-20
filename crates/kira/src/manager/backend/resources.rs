pub(crate) mod clocks;
pub(crate) mod mixer;
pub(crate) mod modulators;
pub(crate) mod sounds;
pub(crate) mod spatial_scenes;

use std::sync::Mutex;

use atomic_arena::Controller;
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};

use crate::{
	clock::Clock,
	manager::settings::Capacities,
	modulator::Modulator,
	sound::wrapper::SoundWrapper,
	spatial::scene::SpatialScene,
	track::{Track, TrackBuilder},
};

use self::{
	clocks::Clocks, mixer::Mixer, modulators::Modulators, sounds::Sounds,
	spatial_scenes::SpatialScenes,
};

pub(crate) struct UnusedResourceProducers {
	pub sound: HeapProducer<SoundWrapper>,
	pub sub_track: HeapProducer<Track>,
	pub clock: HeapProducer<Clock>,
	pub spatial_scene: HeapProducer<SpatialScene>,
	pub modulator: HeapProducer<Box<dyn Modulator>>,
}

pub(crate) struct UnusedResourceConsumers {
	pub sound: Mutex<HeapConsumer<SoundWrapper>>,
	pub sub_track: Mutex<HeapConsumer<Track>>,
	pub clock: Mutex<HeapConsumer<Clock>>,
	pub spatial_scene: Mutex<HeapConsumer<SpatialScene>>,
	pub modulator: Mutex<HeapConsumer<Box<dyn Modulator>>>,
}

pub(crate) fn create_unused_resource_channels(
	capacities: Capacities,
) -> (UnusedResourceProducers, UnusedResourceConsumers) {
	let (unused_sound_wrapper_producer, unused_sound_consumer) =
		HeapRb::new(capacities.sound_capacity).split();
	let (unused_sub_track_producer, unused_sub_track_consumer) =
		HeapRb::new(capacities.sub_track_capacity).split();
	let (unused_clock_producer, unused_clock_consumer) =
		HeapRb::new(capacities.clock_capacity).split();
	let (unused_spatial_scene_producer, unused_spatial_scene_consumer) =
		HeapRb::new(capacities.spatial_scene_capacity).split();
	let (unused_modulator_producer, unused_modulator_consumer) =
		HeapRb::new(capacities.modulator_capacity).split();
	(
		UnusedResourceProducers {
			sound: unused_sound_wrapper_producer,
			sub_track: unused_sub_track_producer,
			clock: unused_clock_producer,
			spatial_scene: unused_spatial_scene_producer,
			modulator: unused_modulator_producer,
		},
		UnusedResourceConsumers {
			sound: Mutex::new(unused_sound_consumer),
			sub_track: Mutex::new(unused_sub_track_consumer),
			clock: Mutex::new(unused_clock_consumer),
			spatial_scene: Mutex::new(unused_spatial_scene_consumer),
			modulator: Mutex::new(unused_modulator_consumer),
		},
	)
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
	unused_resource_producers: UnusedResourceProducers,
	sample_rate: u32,
) -> (Resources, ResourceControllers) {
	let sounds = Sounds::new(capacities.sound_capacity, unused_resource_producers.sound);
	let sound_controller = sounds.controller();
	let mixer = Mixer::new(
		capacities.sub_track_capacity,
		unused_resource_producers.sub_track,
		sample_rate,
		main_track_builder,
	);
	let sub_track_controller = mixer.sub_track_controller();
	let clocks = Clocks::new(capacities.clock_capacity, unused_resource_producers.clock);
	let clock_controller = clocks.controller();
	let spatial_scenes = SpatialScenes::new(
		capacities.spatial_scene_capacity,
		unused_resource_producers.spatial_scene,
	);
	let spatial_scene_controller = spatial_scenes.controller();
	let modulators = Modulators::new(
		capacities.modulator_capacity,
		unused_resource_producers.modulator,
	);
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
	)
}
