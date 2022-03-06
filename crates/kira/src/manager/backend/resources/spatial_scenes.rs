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
		self.remove_unused_scenes();
		for (_, scene) in &mut self.scenes {
			scene.on_start_processing();
		}
	}

	pub fn remove_unused_scenes(&mut self) {
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
			SpatialSceneCommand::Add(id, scene) => self
				.scenes
				.insert_with_key(id.0, scene)
				.expect("Spatial scene arena is full"),
			SpatialSceneCommand::AddEmitter(id, emitter) => {
				if let Some(scene) = self.scenes.get_mut(id.scene().0) {
					scene.add_emitter(id, emitter);
				}
			}
			SpatialSceneCommand::AddListener(id, listener) => {
				if let Some(scene) = self.scenes.get_mut(id.scene().0) {
					scene.add_listener(id, listener);
				}
			}
		}
	}
}
