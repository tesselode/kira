pub(crate) mod clocks;
pub(crate) mod mixer;
pub(crate) mod modulators;
pub(crate) mod sounds;
pub(crate) mod spatial_scenes;

#[cfg(test)]
mod test;

use std::sync::Mutex;

use crate::{
	arena::{Arena, Controller, Key},
	ResourceLimitReached,
};
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};

use crate::{
	clock::Clock,
	manager::settings::Capacities,
	modulator::Modulator,
	sound::Sound,
	spatial::scene::SpatialScene,
	track::{Track, TrackBuilder, TrackHandle},
};

use self::{
	clocks::Clocks, mixer::Mixer, modulators::Modulators, sounds::Sounds,
	spatial_scenes::SpatialScenes,
};

pub(crate) struct ResourceStorage<T> {
	pub(crate) resources: Arena<T>,
	new_resource_consumer: HeapConsumer<(Key, T)>,
	unused_resource_producer: HeapProducer<T>,
}

impl<T> ResourceStorage<T> {
	#[must_use]
	pub fn new(capacity: u16) -> (Self, ResourceController<T>) {
		let (new_resource_producer, new_resource_consumer) = HeapRb::new(capacity as usize).split();
		let (unused_resource_producer, unused_resource_consumer) =
			HeapRb::new(capacity as usize).split();
		let resources = Arena::new(capacity);
		let arena_controller = resources.controller();
		(
			Self {
				resources,
				new_resource_consumer,
				unused_resource_producer,
			},
			ResourceController {
				arena_controller,
				new_resource_producer: Mutex::new(new_resource_producer),
				unused_resource_consumer: Mutex::new(unused_resource_consumer),
			},
		)
	}

	pub fn remove_and_add(&mut self, remove_test: impl FnMut(&T) -> bool) {
		for (_, resource) in self.resources.drain_filter(remove_test) {
			self.unused_resource_producer
				.push(resource)
				.unwrap_or_else(|_| panic!("unused resource producer is full"));
		}
		while let Some((key, resource)) = self.new_resource_consumer.pop() {
			self.resources
				.insert_with_key(key, resource)
				.expect("error inserting resource");
		}
	}

	#[must_use]
	pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
		self.resources.get_mut(key)
	}

	#[must_use]
	pub fn iter_mut(&mut self) -> crate::arena::iter::IterMut<T> {
		self.resources.iter_mut()
	}
}

impl<'a, T> IntoIterator for &'a mut ResourceStorage<T> {
	type Item = (Key, &'a mut T);

	type IntoIter = crate::arena::iter::IterMut<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

pub(crate) struct SelfReferentialResourceStorage<T> {
	pub(crate) resources: Arena<T>,
	keys: Vec<Key>,
	new_resource_consumer: HeapConsumer<(Key, T)>,
	unused_resource_producer: HeapProducer<T>,
	dummy: T,
}

impl<T> SelfReferentialResourceStorage<T> {
	pub fn new(capacity: u16) -> (Self, ResourceController<T>)
	where
		T: Default,
	{
		let (new_resource_producer, new_resource_consumer) = HeapRb::new(capacity as usize).split();
		let (unused_resource_producer, unused_resource_consumer) =
			HeapRb::new(capacity as usize).split();
		let resources = Arena::new(capacity);
		let arena_controller = resources.controller();
		(
			Self {
				resources,
				keys: Vec::with_capacity(capacity as usize),
				new_resource_consumer,
				unused_resource_producer,
				dummy: T::default(),
			},
			ResourceController {
				arena_controller,
				new_resource_producer: Mutex::new(new_resource_producer),
				unused_resource_consumer: Mutex::new(unused_resource_consumer),
			},
		)
	}

	pub fn remove_and_add(&mut self, remove_test: impl FnMut(&T) -> bool) {
		self.remove_unused(remove_test);
		while let Some((key, resource)) = self.new_resource_consumer.pop() {
			self.resources
				.insert_with_key(key, resource)
				.expect("error inserting resource");
			self.keys.push(key);
		}
	}

	#[must_use]
	pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
		self.resources.get_mut(key)
	}

	#[must_use]
	pub fn iter_mut(&mut self) -> crate::arena::iter::IterMut<T> {
		self.resources.iter_mut()
	}

	pub fn for_each(&mut self, mut f: impl FnMut(&mut T, &mut Arena<T>)) {
		for key in &self.keys {
			std::mem::swap(&mut self.resources[*key], &mut self.dummy);
			f(&mut self.dummy, &mut self.resources);
			std::mem::swap(&mut self.resources[*key], &mut self.dummy);
		}
	}

	pub fn for_each_rev(&mut self, mut f: impl FnMut(&mut T, &mut Arena<T>)) {
		for key in self.keys.iter().rev() {
			std::mem::swap(&mut self.resources[*key], &mut self.dummy);
			f(&mut self.dummy, &mut self.resources);
			std::mem::swap(&mut self.resources[*key], &mut self.dummy);
		}
	}

	fn remove_unused(&mut self, mut remove_test: impl FnMut(&T) -> bool) {
		let mut i = 0;
		while i < self.keys.len() && !self.unused_resource_producer.is_full() {
			let key = self.keys[i];
			let resource = &mut self.resources[key];
			if remove_test(resource) {
				let resource = self.resources.remove(key).unwrap();
				self.unused_resource_producer
					.push(resource)
					.unwrap_or_else(|_| panic!("unused resource producer is full"));
				self.keys.remove(i);
			} else {
				i += 1;
			}
		}
	}
}

impl<'a, T> IntoIterator for &'a mut SelfReferentialResourceStorage<T> {
	type Item = (Key, &'a mut T);

	type IntoIter = crate::arena::iter::IterMut<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

pub(crate) struct ResourceController<T> {
	pub arena_controller: Controller,
	pub new_resource_producer: Mutex<HeapProducer<(Key, T)>>,
	pub unused_resource_consumer: Mutex<HeapConsumer<T>>,
}

impl<T> ResourceController<T> {
	pub fn insert(&mut self, resource: T) -> Result<Key, ResourceLimitReached> {
		let key = self.try_reserve()?;
		self.insert_with_key(key, resource);
		Ok(key)
	}

	pub fn try_reserve(&self) -> Result<Key, ResourceLimitReached> {
		self.arena_controller
			.try_reserve()
			.map_err(|_| ResourceLimitReached)
	}

	pub fn insert_with_key(&mut self, key: Key, resource: T) {
		self.remove_unused();
		self.new_resource_producer
			.get_mut()
			.expect("new resource producer mutex poisoned")
			.push((key, resource))
			.unwrap_or_else(|_| panic!("new resource producer full"));
	}

	fn remove_unused(&mut self) {
		let unused_resource_consumer = &mut self
			.unused_resource_consumer
			.get_mut()
			.expect("unused resource consumer mutex poisoned");
		while unused_resource_consumer.pop().is_some() {}
	}

	#[must_use]
	pub fn capacity(&self) -> u16 {
		self.arena_controller.capacity()
	}

	#[must_use]
	pub fn len(&self) -> u16 {
		self.arena_controller.len()
	}
}

pub(crate) struct Resources {
	pub sounds: Sounds,
	pub mixer: Mixer,
	pub clocks: Clocks,
	pub spatial_scenes: SpatialScenes,
	pub modulators: Modulators,
}

pub(crate) struct ResourceControllers {
	pub sound_controller: ResourceController<Box<dyn Sound>>,
	pub sub_track_controller: ResourceController<Track>,
	pub clock_controller: ResourceController<Clock>,
	pub spatial_scene_controller: ResourceController<SpatialScene>,
	pub modulator_controller: ResourceController<Box<dyn Modulator>>,
	pub main_track_handle: TrackHandle,
}

pub(crate) fn create_resources(
	capacities: Capacities,
	main_track_builder: TrackBuilder,
	sample_rate: u32,
) -> (Resources, ResourceControllers) {
	let (sounds, sound_controller) = Sounds::new(capacities.sound_capacity);
	let (mixer, sub_track_controller, main_track_handle) = Mixer::new(
		capacities.sub_track_capacity,
		sample_rate,
		main_track_builder,
	);
	let (clocks, clock_controller) = Clocks::new(capacities.clock_capacity);
	let (spatial_scenes, spatial_scene_controller) =
		SpatialScenes::new(capacities.spatial_scene_capacity);
	let (modulators, modulator_controller) = Modulators::new(capacities.modulator_capacity);
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
			main_track_handle,
		},
	)
}
