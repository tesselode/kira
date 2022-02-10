use crate::dsp::Frame;

use super::{Backend, Renderer};

enum State {
	Uninitialized,
	Initialized { renderer: Renderer },
}

/// A backend that does not connect to any lower-level
/// audio APIs, but allows manually calling
/// [`Renderer::on_start_processing`] and [`Renderer::process`].
///
/// This is useful for testing and benchmarking.
pub struct MockBackend {
	sample_rate: u32,
	state: State,
}

impl MockBackend {
	/// Creates a new [`MockBackend`].
	pub fn new(sample_rate: u32) -> Self {
		Self {
			sample_rate,
			state: State::Uninitialized,
		}
	}

	/// Calls the [`on_start_processing`](Renderer::on_start_processing)
	/// callback of the [`Renderer`].
	pub fn on_start_processing(&mut self) {
		if let State::Initialized { renderer } = &mut self.state {
			renderer.on_start_processing();
		} else {
			panic!("backend is not initialized")
		}
	}

	/// Calls the [`process`](Renderer::process) callback of the [`Renderer`].
	pub fn process(&mut self) -> Frame {
		if let State::Initialized { renderer } = &mut self.state {
			renderer.process()
		} else {
			panic!("backend is not initialized")
		}
	}
}

impl Backend for MockBackend {
	type InitError = ();

	fn sample_rate(&mut self) -> u32 {
		self.sample_rate
	}

	fn init(&mut self, renderer: Renderer) -> Result<(), Self::InitError> {
		self.state = State::Initialized { renderer };
		Ok(())
	}
}
