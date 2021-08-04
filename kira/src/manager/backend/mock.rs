use crate::{
	manager::{renderer::Renderer, resources::UnusedResourceCollector},
	Frame,
};

use super::Backend;

const SAMPLE_RATE: u32 = 48_000;
const BUFFER_LENGTH: usize = 512;

enum State {
	Uninitialized,
	Initialized {
		renderer: Renderer,
		unused_resource_collector: UnusedResourceCollector,
		buffer_start_timer: usize,
	},
}

pub struct MockBackend {
	state: State,
}

impl MockBackend {
	pub fn new() -> Self {
		Self {
			state: State::Uninitialized,
		}
	}

	pub fn on_start_processing(&mut self) {
		if let State::Initialized { renderer, .. } = &mut self.state {
			renderer.on_start_processing();
		} else {
			panic!("backend is not initialized")
		}
	}

	pub fn process(&mut self) -> Frame {
		if let State::Initialized {
			renderer,
			buffer_start_timer,
			..
		} = &mut self.state
		{
			*buffer_start_timer -= 1;
			if *buffer_start_timer == 0 {
				renderer.on_start_processing();
				*buffer_start_timer += BUFFER_LENGTH;
			}
			renderer.process()
		} else {
			panic!("backend is not initialized")
		}
	}

	pub fn collect_unused_resources(&mut self) {
		if let State::Initialized {
			unused_resource_collector,
			..
		} = &mut self.state
		{
			unused_resource_collector.drain();
		} else {
			panic!("backend is not initialized")
		}
	}
}

impl Backend for MockBackend {
	type InitError = ();

	fn sample_rate(&mut self) -> u32 {
		SAMPLE_RATE
	}

	fn init(
		&mut self,
		renderer: Renderer,
		unused_resource_collector: UnusedResourceCollector,
	) -> Result<(), Self::InitError> {
		self.state = State::Initialized {
			renderer,
			unused_resource_collector,
			buffer_start_timer: 1,
		};
		Ok(())
	}
}
