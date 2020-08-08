use crate::project::MetronomeSettings;

pub struct Metronome {
	tempo: f32,
	settings: MetronomeSettings,
	ticking: bool,
	time: f32,
	previous_time: f32,
}

impl Metronome {
	pub fn new(tempo: f32, settings: MetronomeSettings) -> Self {
		Self {
			tempo,
			settings,
			ticking: false,
			time: 0.0,
			previous_time: 0.0,
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

	pub fn update(&mut self, dt: f32, interval_event_collector: &mut Vec<f32>) {
		if !self.ticking {
			return;
		}
		self.previous_time = self.time;
		self.time += (self.tempo / 60.0) * dt;
		for interval in &self.settings.interval_events_to_emit {
			if self.interval_passed(*interval) {
				interval_event_collector.push(*interval);
			}
		}
	}

	pub fn interval_passed(&self, interval: f32) -> bool {
		(self.previous_time % interval) > (self.time % interval)
	}
}
