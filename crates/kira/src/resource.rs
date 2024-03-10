use atomic_arena::{Arena, Controller, Key};
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};

use crate::{clock::Clock, modulator::Modulator};

pub(crate) trait Resource {
	type Id: ResourceId;

	fn on_start_processing(&mut self);

	fn should_be_removed(&self) -> bool;
}

pub(crate) trait ResourceId: Copy {
	fn key(&self) -> Key;
}

pub(crate) struct Resources<T: Resource> {
	pub items: Arena<Option<T>>,
	ids: Vec<T::Id>,
	new_resource_consumer: HeapConsumer<(T::Id, T)>,
	unused_resource_producer: HeapProducer<T>,
}

impl<T: Resource> Resources<T> {
	#[allow(clippy::type_complexity)]
	pub fn new(capacity: usize) -> (Self, Controller, HeapProducer<(T::Id, T)>, HeapConsumer<T>) {
		let items = Arena::new(capacity);
		let (new_resource_producer, new_resource_consumer) = HeapRb::new(capacity).split();
		let (unused_resource_producer, unused_resource_consumer) = HeapRb::new(capacity).split();
		let controller = items.controller();
		(
			Self {
				items,
				ids: Vec::with_capacity(capacity),
				new_resource_consumer,
				unused_resource_producer,
			},
			controller,
			new_resource_producer,
			unused_resource_consumer,
		)
	}

	pub fn items(&self) -> &Arena<Option<T>> {
		&self.items
	}

	pub fn items_mut(&mut self) -> &mut Arena<Option<T>> {
		&mut self.items
	}

	pub fn on_start_processing(&mut self) {
		self.remove_unused();
		while let Some((id, item)) = self.new_resource_consumer.pop() {
			self.items
				.insert_with_key(id.key(), Some(item))
				.expect("resource arena is full");
			self.ids.push(id);
		}
		for (_, item) in &mut self.items {
			item.as_mut().unwrap().on_start_processing();
		}
	}

	pub fn for_each(&mut self, mut f: impl FnMut(&mut T, &mut Arena<Option<T>>)) {
		for id in self.ids.iter().rev() {
			let mut item = self.items[id.key()].take().unwrap();
			f(&mut item, &mut self.items);
			self.items[id.key()] = Some(item);
		}
	}

	fn remove_unused(&mut self) {
		let mut i = 0;
		while i < self.ids.len() && !self.unused_resource_producer.is_full() {
			let id = self.ids[i];
			let item = self.items[id.key()].as_mut().unwrap();
			if item.should_be_removed() {
				if self
					.unused_resource_producer
					.push(self.items.remove(id.key()).unwrap().unwrap())
					.is_err()
				{
					panic!("Unused item producer is full")
				}
				self.ids.remove(i);
			} else {
				i += 1;
			}
		}
	}
}

pub type Clocks = Resources<Clock>;
pub type Modulators = Resources<Box<dyn Modulator>>;
