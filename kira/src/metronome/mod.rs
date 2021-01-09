mod handle;
mod metronomes;
mod settings;

use flume::Sender;
use uuid::Uuid;

use crate::{parameter::Parameters, tempo::Tempo, util::generate_uuid, value::CachedValue, Value};
pub use handle::MetronomeHandle;
pub(crate) use metronomes::Metronomes;
pub use settings::MetronomeSettings;

/**
A unique identifier for a metronome.

You cannot create this manually - a metronome ID is created
when you create a metronome with an [`AudioManager`](crate::manager::AudioManager).
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
pub struct MetronomeId {
	uuid: Uuid,
}

impl MetronomeId {
	pub(crate) fn new() -> Self {
		Self {
			uuid: generate_uuid(),
		}
	}
}

impl From<&MetronomeHandle> for MetronomeId {
	fn from(handle: &MetronomeHandle) -> Self {
		handle.id()
	}
}

#[derive(Debug, Clone)]
pub(crate) struct Metronome {
	tempo: CachedValue<Tempo>,
	interval_events_to_emit: Vec<f64>,
	ticking: bool,
	time: f64,
	previous_time: f64,
	event_sender: Sender<f64>,
}

impl Metronome {
	pub fn new(settings: MetronomeSettings, event_sender: Sender<f64>) -> Self {
		Self {
			tempo: CachedValue::new(settings.tempo, Tempo(120.0)),
			interval_events_to_emit: settings.interval_events_to_emit,
			ticking: false,
			time: 0.0,
			previous_time: 0.0,
			event_sender,
		}
	}

	pub fn effective_tempo(&self) -> Tempo {
		if self.ticking {
			self.tempo.value()
		} else {
			Tempo(0.0)
		}
	}

	pub fn set_tempo(&mut self, tempo: Value<Tempo>) {
		self.tempo.set(tempo);
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

	pub fn update(&mut self, dt: f64, parameters: &Parameters) {
		self.tempo.update(parameters);
		if self.ticking {
			self.previous_time = self.time;
			self.time += (self.tempo.value().0 / 60.0) * dt;
			for interval in &self.interval_events_to_emit {
				if self.interval_passed(*interval) {
					self.event_sender.try_send(*interval).ok();
				}
			}
		}
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
