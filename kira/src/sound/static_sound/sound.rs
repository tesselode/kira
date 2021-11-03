use crate::{
	dsp::Frame,
	manager::resources::{Clocks, Parameters},
	sound::Sound,
	track::TrackId,
};

use super::data::StaticSoundData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaybackState {
	Playing,
	Pausing,
	Stopped,
}

pub struct StaticSound {
	data: StaticSoundData,
	state: PlaybackState,
	position: f64,
}

impl StaticSound {
	pub fn new(data: StaticSoundData) -> Self {
		Self {
			data,
			state: PlaybackState::Playing,
			position: 0.0,
		}
	}
}

impl Sound for StaticSound {
	fn sample_rate(&mut self) -> u32 {
		self.data.sample_rate
	}

	fn track(&mut self) -> TrackId {
		TrackId::Main
	}

	fn process(&mut self, dt: f64, _parameters: &Parameters, _clocks: &Clocks) -> Frame {
		let out = self.data.frame_at_position(self.position);
		if self.position > self.data.duration().as_secs_f64() {
			self.state = PlaybackState::Stopped;
		}
		self.position += dt;
		out
	}

	fn finished(&self) -> bool {
		self.state == PlaybackState::Stopped
	}
}
