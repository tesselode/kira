use std::ops::{Deref, DerefMut};

use crate::manager::backend::Renderer;
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};

/// Wraps a [`Renderer`] so that when it's dropped,
/// it gets sent back through a thread channel.
///
/// This allows us to retrieve the renderer after a closure
/// that takes ownership of the [`Renderer`] is dropped
/// because of a cpal error.
pub(super) struct RendererWrapper {
	renderer: Option<Renderer>,
	producer: HeapProducer<Renderer>,
}

impl RendererWrapper {
	pub(super) fn new(renderer: Renderer) -> (Self, HeapConsumer<Renderer>) {
		let (producer, consumer) = HeapRb::new(1).split();
		(
			Self {
				renderer: Some(renderer),
				producer,
			},
			consumer,
		)
	}
}

impl Deref for RendererWrapper {
	type Target = Renderer;

	fn deref(&self) -> &Self::Target {
		self.renderer.as_ref().unwrap()
	}
}

impl DerefMut for RendererWrapper {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.renderer.as_mut().unwrap()
	}
}

impl Drop for RendererWrapper {
	fn drop(&mut self) {
		if self
			.producer
			.push(self.renderer.take().expect("The renderer does not exist"))
			.is_err()
		{
			panic!("The renderer producer is full");
		}
	}
}
