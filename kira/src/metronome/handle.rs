use basedrop::Shared;
use ringbuf::Consumer;

use crate::Tempo;

use super::MetronomeState;

pub struct MetronomeHandle {
	metronome_state: Shared<MetronomeState>,
	interval_event_consumer: Consumer<f64>,
}

impl MetronomeHandle {
	pub(crate) fn new(
		metronome_state: Shared<MetronomeState>,
		interval_event_consumer: Consumer<f64>,
	) -> Self {
		Self {
			metronome_state,
			interval_event_consumer,
		}
	}

	pub fn tempo(&self) -> Tempo {
		self.metronome_state.tempo()
	}

	pub fn ticking(&self) -> bool {
		self.metronome_state.ticking()
	}

	pub fn time(&self) -> f64 {
		self.metronome_state.time()
	}

	pub fn set_tempo(&self, tempo: Tempo) {
		self.metronome_state.set_tempo(tempo);
	}

	pub fn start(&self) {
		self.metronome_state.start();
	}

	pub fn pause(&self) {
		self.metronome_state.pause();
	}

	pub fn stop(&self) {
		self.metronome_state.stop();
	}

	pub fn pop_event(&mut self) -> Option<f64> {
		self.interval_event_consumer.pop()
	}
}
