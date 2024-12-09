use std::ops::{Deref, DerefMut};

use rtrb::{Consumer, Producer, RingBuffer};

pub struct SendOnDrop<T> {
	item: Option<T>,
	producer: Producer<T>,
}

impl<T> SendOnDrop<T> {
	pub(super) fn new(item: T) -> (Self, Consumer<T>) {
		let (producer, consumer) = RingBuffer::new(1);
		(
			Self {
				item: Some(item),
				producer,
			},
			consumer,
		)
	}
}

impl<T> Deref for SendOnDrop<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.item.as_ref().unwrap()
	}
}

impl<T> DerefMut for SendOnDrop<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.item.as_mut().unwrap()
	}
}

impl<T> Drop for SendOnDrop<T> {
	fn drop(&mut self) {
		if self
			.producer
			.push(self.item.take().expect("The item does not exist"))
			.is_err()
		{
			panic!("The item producer is full");
		}
	}
}
