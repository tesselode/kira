use atomic_arena::{Arena, Controller};
use ringbuf::HeapProducer;

use crate::{manager::command::SpatialSceneCommand, spatial::scene::SpatialScene};

pub(crate) struct SpatialScenes {
	scenes: Arena<SpatialScene>,
	unused_scene_producer: HeapProducer<SpatialScene>,
}

impl SpatialScenes {
	pub fn new(capacity: usize, unused_scene_producer: HeapProducer<SpatialScene>) -> Self {
		Self {
			scenes: Arena::new(capacity),
			unused_scene_producer,
		}
	}

	pub fn controller(&self) -> Controller {
		self.scenes.controller()
	}

	pub fn on_start_processing(&mut self) {
		if self.unused_scene_producer.is_full() {
			return;
		}
		for (_, scene) in self
			.scenes
			.drain_filter(|scene| scene.shared().is_marked_for_removal())
		{
			if self.unused_scene_producer.push(scene).is_err() {
				panic!("Unused scene producer is full")
			}
			if self.unused_scene_producer.is_full() {
				return;
			}
		}
	}

	pub fn run_command(&mut self, command: SpatialSceneCommand) {
		match command {
			SpatialSceneCommand::Add(key, scene) => self
				.scenes
				.insert_with_key(key.0, scene)
				.expect("Spatial scene arena is full"),
		}
	}
}
