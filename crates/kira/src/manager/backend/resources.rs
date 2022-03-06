pub(crate) mod clocks;
pub(crate) mod mixer;
pub(crate) mod sounds;
pub(crate) mod spatial_scenes;

use atomic_arena::Controller;
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};

use crate::{
	clock::Clock,
	manager::settings::Capacities,
	sound::Sound,
	spatial::scene::SpatialScene,
	track::{Track, TrackBuilder},
};

use self::{clocks::Clocks, mixer::Mixer, sounds::Sounds, spatial_scenes::SpatialScenes};

pub(crate) struct UnusedResourceProducers {
	pub sound: HeapProducer<Box<dyn Sound>>,
	pub sub_track: HeapProducer<Track>,
	pub clock: HeapProducer<Clock>,
	pub spatial_scene: HeapProducer<SpatialScene>,
}

pub(crate) struct UnusedResourceConsumers {
	pub sound: HeapConsumer<Box<dyn Sound>>,
	pub sub_track: HeapConsumer<Track>,
	pub clock: HeapConsumer<Clock>,
	pub spatial_scene: HeapConsumer<SpatialScene>,
}

pub(crate) fn create_unused_resource_channels(
	capacities: Capacities,
) -> (UnusedResourceProducers, UnusedResourceConsumers) {
	let (unused_sound_producer, unused_sound_consumer) =
		HeapRb::new(capacities.sound_capacity).split();
	let (unused_sub_track_producer, unused_sub_track_consumer) =
		HeapRb::new(capacities.sub_track_capacity).split();
	let (unused_clock_producer, unused_clock_consumer) =
		HeapRb::new(capacities.clock_capacity).split();
	let (unused_spatial_scene_producer, unused_spatial_scene_consumer) =
		HeapRb::new(capacities.spatial_scene_capacity).split();
	(
		UnusedResourceProducers {
			sound: unused_sound_producer,
			sub_track: unused_sub_track_producer,
			clock: unused_clock_producer,
			spatial_scene: unused_spatial_scene_producer,
		},
		UnusedResourceConsumers {
			sound: unused_sound_consumer,
			sub_track: unused_sub_track_consumer,
			clock: unused_clock_consumer,
			spatial_scene: unused_spatial_scene_consumer,
		},
	)
}

pub(crate) struct Resources {
	pub sounds: Sounds,
	pub mixer: Mixer,
	pub clocks: Clocks,
	pub spatial_scenes: SpatialScenes,
}

pub(crate) struct ResourceControllers {
	pub sound_controller: Controller,
	pub sub_track_controller: Controller,
	pub clock_controller: Controller,
	pub spatial_scene_controller: Controller,
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
	(
		Resources {
			sounds,
			mixer,
			clocks,
			spatial_scenes,
		},
		ResourceControllers {
			sound_controller,
			sub_track_controller,
			clock_controller,
			spatial_scene_controller,
		},
	)
}
