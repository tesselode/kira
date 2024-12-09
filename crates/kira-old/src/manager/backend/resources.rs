pub(crate) mod clocks;
pub(crate) mod listeners;
pub(crate) mod mixer;
pub(crate) mod modulators;

#[cfg(test)]
mod test;

use std::{
	fmt::{Debug, Formatter},
	sync::Mutex,
};

use crate::{
	arena::{Arena, Controller, Key},
	listener::Listener,
	track::{MainTrackBuilder, MainTrackHandle, SendTrack, Track},
	ResourceLimitReached,
};
use listeners::Listeners;
use rtrb::{Consumer, Producer, RingBuffer};

use crate::{clock::Clock, manager::settings::Capacities, modulator::Modulator};

use self::{clocks::Clocks, mixer::Mixer, modulators::Modulators};

pub(crate) struct ResourceStorage<T> {
	pub(crate) resources: Arena<T>,
	new_resource_consumer: Consumer<(Key, T)>,
	unused_resource_producer: Producer<T>,
}

impl<T> ResourceStorage<T> {
	#[must_use]
	pub fn new(capacity: u16) -> (Self, ResourceController<T>) {
		let (new_resource_producer, new_resource_consumer) = RingBuffer::new(capacity as usize);
		let (unused_resource_producer, unused_resource_consumer) =
			RingBuffer::new(capacity as usize);
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
		while let Ok((key, resource)) = self.new_resource_consumer.pop() {
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
	pub fn iter(&self) -> crate::arena::iter::Iter<T> {
		self.resources.iter()
	}

	#[must_use]
	pub fn iter_mut(&mut self) -> crate::arena::iter::IterMut<T> {
		self.resources.iter_mut()
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.resources.is_empty()
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
	new_resource_consumer: Consumer<(Key, T)>,
	unused_resource_producer: Producer<T>,
	dummy: T,
}

impl<T> SelfReferentialResourceStorage<T> {
	pub fn new(capacity: u16) -> (Self, ResourceController<T>)
	where
		T: Default,
	{
		let (new_resource_producer, new_resource_consumer) = RingBuffer::new(capacity as usize);
		let (unused_resource_producer, unused_resource_consumer) =
			RingBuffer::new(capacity as usize);
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
		while let Ok((key, resource)) = self.new_resource_consumer.pop() {
			self.resources
				.insert_with_key(key, resource)
				.expect("error inserting resource");
			self.keys.push(key);
		}
	}

	#[cfg(test)]
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
	pub new_resource_producer: Mutex<Producer<(Key, T)>>,
	pub unused_resource_consumer: Mutex<Consumer<T>>,
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
		while unused_resource_consumer.pop().is_ok() {}
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

impl<T> Debug for ResourceController<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ResourceController")
			.field("arena_controller", &self.arena_controller)
			.field("new_resource_producer", &ProducerDebug)
			.field("unused_resource_consumer", &ConsumerDebug)
			.finish()
	}
}

struct ProducerDebug;

impl Debug for ProducerDebug {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Producer").finish()
	}
}

struct ConsumerDebug;

impl Debug for ConsumerDebug {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Consumer").finish()
	}
}

pub(crate) struct Resources {
	pub mixer: Mixer,
	pub clocks: Clocks,
	pub modulators: Modulators,
	pub listeners: Listeners,
}

pub(crate) struct ResourceControllers {
	pub sub_track_controller: ResourceController<Track>,
	pub send_track_controller: ResourceController<SendTrack>,
	pub clock_controller: ResourceController<Clock>,
	pub modulator_controller: ResourceController<Box<dyn Modulator>>,
	pub listener_controller: ResourceController<Listener>,
	pub main_track_handle: MainTrackHandle,
}

pub(crate) fn create_resources(
	capacities: Capacities,
	main_track_builder: MainTrackBuilder,
	sample_rate: u32,
) -> (Resources, ResourceControllers) {
	let (mixer, sub_track_controller, send_track_controller, main_track_handle) = Mixer::new(
		capacities.sub_track_capacity,
		capacities.send_track_capacity,
		sample_rate,
		main_track_builder,
	);
	let (clocks, clock_controller) = Clocks::new(capacities.clock_capacity);
	let (modulators, modulator_controller) = Modulators::new(capacities.modulator_capacity);
	let (listeners, listener_controller) = Listeners::new(capacities.listener_capacity);
	(
		Resources {
			mixer,
			clocks,
			modulators,
			listeners,
		},
		ResourceControllers {
			sub_track_controller,
			send_track_controller,
			clock_controller,
			modulator_controller,
			listener_controller,
			main_track_handle,
		},
	)
}
