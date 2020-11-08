use crate::{command::MetronomeCommand, tempo::Tempo};
use std::vec::Drain;

#[derive(Debug, Clone)]
/// Settings for a metronome.
pub struct MetronomeSettings {
	/// The tempo of the metronome (in beats per minute).
	pub tempo: Tempo,
	/// Which intervals (in beats) the metronome should emit events for.
	///
	/// For example, if this is set to `vec![0.25, 0.5, 1.0]`, then
	/// the audio manager will receive `MetronomeIntervalPassed` events
	/// every quarter of a beat, half of a beat, and beat.
	pub interval_events_to_emit: Vec<f64>,
}

impl Default for MetronomeSettings {
	fn default() -> Self {
		Self {
			tempo: Tempo(120.0),
			interval_events_to_emit: vec![],
		}
	}
}

#[derive(Debug, Clone)]
pub(crate) struct Metronome {
	pub settings: MetronomeSettings,
	ticking: bool,
	time: f64,
	previous_time: f64,
	interval_event_queue: Vec<f64>,
}

impl Metronome {
	pub fn new(settings: MetronomeSettings) -> Self {
		let num_interval_events = settings.interval_events_to_emit.len();
		Self {
			settings,
			ticking: false,
			time: 0.0,
			previous_time: 0.0,
			interval_event_queue: Vec::with_capacity(num_interval_events),
		}
	}

	pub fn effective_tempo(&self) -> Tempo {
		if self.ticking {
			self.settings.tempo
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
			MetronomeCommand::SetMetronomeTempo(tempo) => self.settings.tempo = tempo,
			MetronomeCommand::StartMetronome => self.start(),
			MetronomeCommand::PauseMetronome => self.pause(),
			MetronomeCommand::StopMetronome => self.stop(),
		}
	}

	pub fn update(&mut self, dt: f64) -> Drain<f64> {
		if self.ticking {
			self.previous_time = self.time;
			self.time += (self.settings.tempo.0 / 60.0) * dt;
			for interval in &self.settings.interval_events_to_emit {
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
