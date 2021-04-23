pub mod handle;
pub mod settings;

use std::sync::atomic::AtomicBool;

use atomig::{Atomic, Ordering};
use basedrop::Shared;
use ringbuf::Producer;

use crate::{value::Value, Tempo};

pub(crate) struct MetronomeState {
	tempo: Value<Tempo>,
	ticking: AtomicBool,
	time: Atomic<f64>,
	previous_time: Atomic<f64>,
}

impl MetronomeState {
	pub fn new(tempo: Value<Tempo>) -> Self {
		Self {
			tempo,
			ticking: AtomicBool::new(false),
			time: Atomic::new(0.0),
			previous_time: Atomic::new(0.0),
		}
	}

	pub fn tempo(&self) -> Tempo {
		self.tempo.get()
	}

	pub fn effective_tempo(&self) -> Tempo {
		if self.ticking() {
			self.tempo()
		} else {
			Tempo(0.0)
		}
	}

	pub fn ticking(&self) -> bool {
		self.ticking.load(Ordering::SeqCst)
	}

	pub fn time(&self) -> f64 {
		self.time.load(Ordering::SeqCst)
	}

	pub fn interval_passed(&self, interval: f64) -> bool {
		if !self.ticking() {
			return false;
		}
		let previous_time = self.previous_time.load(Ordering::SeqCst);
		let time = self.time();
		if previous_time == 0.0 {
			return true;
		}
		(previous_time % interval) > (time % interval)
	}

	pub fn start(&self) {
		self.ticking.store(true, Ordering::SeqCst);
	}

	pub fn pause(&self) {
		self.ticking.store(false, Ordering::SeqCst);
	}

	pub fn stop(&self) {
		self.ticking.store(false, Ordering::SeqCst);
		self.time.store(0.0, Ordering::SeqCst);
		self.previous_time.store(0.0, Ordering::SeqCst);
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
		self.state.previous_time.store(time, Ordering::SeqCst);
		self.state
			.time
			.store(time + tempo.0 / 60.0 * dt, Ordering::SeqCst);
		for interval in &self.interval_events_to_emit {
			if self.state.interval_passed(*interval) {
				self.interval_event_producer.push(*interval).ok();
			}
		}
	}
}

#[cfg(feature = "log_drops")]
impl Drop for Metronome {
	fn drop(&mut self) {
		println!("dropped metronome");
	}
}
