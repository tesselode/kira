pub mod handle;
pub mod settings;

use std::sync::atomic::AtomicBool;

use atomig::{Atomic, Ordering};
use basedrop::Shared;
use ringbuf::Producer;

use crate::Tempo;

pub(crate) struct MetronomeState {
	tempo: Atomic<Tempo>,
	ticking: AtomicBool,
	time: Atomic<f64>,
	previous_time: Atomic<f64>,
}

impl MetronomeState {
	pub fn new(tempo: Tempo) -> Self {
		Self {
			tempo: Atomic::new(tempo),
			ticking: AtomicBool::new(false),
			time: Atomic::new(0.0),
			previous_time: Atomic::new(0.0),
		}
	}

	pub fn tempo(&self) -> Tempo {
		self.tempo.load(Ordering::Relaxed)
	}

	pub fn effective_tempo(&self) -> Tempo {
		if self.ticking() {
			self.tempo()
		} else {
			Tempo(0.0)
		}
	}

	pub fn ticking(&self) -> bool {
		self.ticking.load(Ordering::Relaxed)
	}

	pub fn time(&self) -> f64 {
		self.time.load(Ordering::Relaxed)
	}

	pub fn interval_passed(&self, interval: f64) -> bool {
		if !self.ticking() {
			return false;
		}
		let previous_time = self.previous_time.load(Ordering::Relaxed);
		let time = self.time();
		if previous_time == 0.0 {
			return true;
		}
		(previous_time % interval) > (time % interval)
	}

	pub fn start(&self) {
		self.ticking.store(true, Ordering::Relaxed);
	}

	pub fn pause(&self) {
		self.ticking.store(false, Ordering::Relaxed);
	}

	pub fn stop(&self) {
		self.ticking.store(false, Ordering::Relaxed);
		self.time.store(0.0, Ordering::Relaxed);
		self.previous_time.store(0.0, Ordering::Relaxed);
	}

	pub fn set_tempo(&self, tempo: Tempo) {
		self.tempo.store(tempo, Ordering::Relaxed);
	}
}

pub(crate) struct Metronome {
	state: Shared<MetronomeState>,
	interval_events_to_emit: Vec<f64>,
	interval_event_producer: Producer<f64>,
}

impl Metronome {
	pub fn new(
		state: Shared<MetronomeState>,
		interval_events_to_emit: Vec<f64>,
		interval_event_producer: Producer<f64>,
	) -> Self {
		Self {
			state,
			interval_events_to_emit,
			interval_event_producer,
		}
	}

	pub fn update(&mut self, dt: f64) {
		let time = self.state.time();
		let tempo = self.state.tempo();
		self.state.previous_time.store(time, Ordering::Relaxed);
		self.state
			.time
			.store(time + tempo.0 / 60.0 * dt, Ordering::Relaxed);
		for interval in &self.interval_events_to_emit {
			if self.state.interval_passed(*interval) {
				self.interval_event_producer.push(*interval).ok();
			}
		}
	}
}
