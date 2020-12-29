mod settings;

pub use settings::MetronomeSettings;

use crate::{command::MetronomeCommand, parameter::Parameters, tempo::Tempo, value::CachedValue};
use std::vec::Drain;

#[derive(Debug, Clone)]
pub(crate) struct Metronome {
	tempo: CachedValue<Tempo>,
	interval_events_to_emit: Vec<f64>,
	ticking: bool,
	time: f64,
	previous_time: f64,
	interval_event_queue: Vec<f64>,
}

impl Metronome {
	pub fn new(settings: MetronomeSettings) -> Self {
		let num_interval_events = settings.interval_events_to_emit.len();
		Self {
			tempo: CachedValue::new(settings.tempo, Tempo(120.0)),
			interval_events_to_emit: settings.interval_events_to_emit,
			ticking: false,
			time: 0.0,
			previous_time: 0.0,
			interval_event_queue: Vec::with_capacity(num_interval_events),
		}
	}

	pub fn effective_tempo(&self) -> Tempo {
		if self.ticking {
			self.tempo.value()
		} else {
			Tempo(0.0)
		}
	}

	pub fn start(&mut self) {
		self.ticking = true;
	}

	pub fn pause(&mut self) {
		self.ticking = false;
	}

	pub fn stop(&mut self) {
		self.ticking = false;
		self.time = 0.0;
		self.previous_time = 0.0;
	}

	pub fn run_command(&mut self, command: MetronomeCommand) {
		match command {
			MetronomeCommand::SetMetronomeTempo(tempo) => self.tempo.set(tempo),
			MetronomeCommand::StartMetronome => self.start(),
			MetronomeCommand::PauseMetronome => self.pause(),
			MetronomeCommand::StopMetronome => self.stop(),
		}
	}

	pub fn update(&mut self, dt: f64, parameters: &Parameters) -> Drain<f64> {
		self.tempo.update(parameters);
		if self.ticking {
			self.previous_time = self.time;
			self.time += (self.tempo.value().0 / 60.0) * dt;
			for interval in &self.interval_events_to_emit {
				if self.interval_passed(*interval) {
					self.interval_event_queue.push(*interval);
				}
			}
		}
		self.interval_event_queue.drain(..)
	}

	pub fn interval_passed(&self, interval: f64) -> bool {
		if !self.ticking {
			return false;
		}
		if self.previous_time == 0.0 {
			return true;
		}
		(self.previous_time % interval) > (self.time % interval)
	}
}
