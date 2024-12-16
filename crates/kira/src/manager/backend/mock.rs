//! Useful for testing and benchmarking.

use std::sync::Mutex;

use super::{Backend, Renderer};

enum State {
	Uninitialized,
	Initialized { renderer: Mutex<Renderer> },
}

/// Settings for the mock backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MockBackendSettings {
	/// The sample rate that the [`Renderer`] should run at.
	pub sample_rate: u32,
}

impl Default for MockBackendSettings {
	fn default() -> Self {
		Self { sample_rate: 1 }
	}
}

/// A backend that does not connect to any lower-level
/// audio APIs, but allows manually calling
/// [`Renderer::on_start_processing`] and [`Renderer::process`].
///
/// This is useful for testing and benchmarking.
pub struct MockBackend {
	sample_rate: u32,
	state: State,
	frames: Vec<f32>,
}

impl MockBackend {
	/// Changes the sample rate of the [`Renderer`].
	pub fn set_sample_rate(&mut self, sample_rate: u32) {
		self.sample_rate = sample_rate;
		if let State::Initialized { renderer } = &mut self.state {
			renderer
				.get_mut()
				.expect("mutex poisoned")
				.on_change_sample_rate(sample_rate);
		}
	}

	/// Calls the [`on_start_processing`](Renderer::on_start_processing)
	/// callback of the [`Renderer`].
	pub fn on_start_processing(&mut self) {
		if let State::Initialized { renderer } = &mut self.state {
			renderer
				.get_mut()
				.expect("mutex poisoned")
				.on_start_processing();
		} else {
			panic!("backend is not initialized")
		}
	}

	/// Calls the [`process`](Renderer::process) callback of the [`Renderer`].
	pub fn process(&mut self) {
		if let State::Initialized { renderer } = &mut self.state {
			renderer
				.get_mut()
				.expect("mutex poisoned")
				.process(&mut self.frames, 2)
		} else {
			panic!("backend is not initialized")
		}
	}
}

impl Backend for MockBackend {
	type Settings = MockBackendSettings;

	type Error = ();

	fn setup(
		settings: Self::Settings,
		internal_buffer_size: usize,
	) -> Result<(Self, u32), Self::Error> {
		Ok((
			Self {
				sample_rate: settings.sample_rate,
				state: State::Uninitialized,
				frames: vec![0.0; internal_buffer_size * 2],
			},
			settings.sample_rate,
		))
	}

	fn start(&mut self, renderer: Renderer) -> Result<(), Self::Error> {
		self.state = State::Initialized {
			renderer: Mutex::new(renderer),
		};
		Ok(())
	}
}
