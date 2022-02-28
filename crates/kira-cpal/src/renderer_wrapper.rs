use std::ops::{Deref, DerefMut};

use kira::manager::backend::Renderer;
use ringbuf::Producer;

/// Wraps a [`Renderer`] so that when it's dropped,
/// it gets sent back thread channel.
///
/// This allows us to retrieve the renderer after a closure
/// that takes ownership of the [`Renderer`] is dropped
/// because of a cpal error.
pub(super) struct RendererWrapper {
	pub renderer: Option<Renderer>,
	pub producer: Producer<Renderer>,
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
