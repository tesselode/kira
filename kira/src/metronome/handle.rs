use basedrop::Shared;
use ringbuf::Consumer;

use crate::Tempo;

use super::MetronomeState;

pub struct MetronomeHandle {
	state: Shared<MetronomeState>,
	interval_event_consumer: Consumer<f64>,
}

impl MetronomeHandle {
	pub(crate) fn new(
		metronome_state: Shared<MetronomeState>,
		interval_event_consumer: Consumer<f64>,
	) -> Self {
		Self {
			state: metronome_state,
			interval_event_consumer,
		}
	}

	pub(crate) fn state(&self) -> Shared<MetronomeState> {
		self.state.clone()
	}

	pub fn tempo(&self) -> Tempo {
		self.state.tempo()
	}

	pub fn ticking(&self) -> bool {
		self.state.ticking()
	}

	pub fn time(&self) -> f64 {
		self.state.time()
	}

	pub fn start(&self) {
		self.state.start();
	}

	pub fn pause(&self) {
		self.state.pause();
	}

	pub fn stop(&self) {
		self.state.stop();
	}

	pub fn pop_event(&mut self) -> Option<f64> {
		self.interval_event_consumer.pop()
	}
}
