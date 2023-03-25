mod error;
mod handle;
mod settings;
mod static_sound;
#[cfg(feature = "symphonia")]
mod symphonia;

use std::collections::VecDeque;

pub use error::*;
pub use handle::*;
pub use settings::*;
pub use static_sound::*;

use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::{interpolate_frame, Frame},
	modulator::value_provider::ModulatorValueProvider,
	track::TrackId,
	OutputDestination,
};

use super::Sound;

trait FiniteSoundData: Send {
	fn sample_rate(&mut self) -> u32;

	fn len(&mut self) -> usize;

	fn frame(&mut self, index: usize) -> Frame;

	fn buffer_len(&mut self) -> usize;
}

struct FiniteSound {
	data: Box<dyn FiniteSoundData>,
	buffer: VecDeque<Frame>,
	playback_state: PlaybackState,
	current_frame_index: usize,
	fractional_playback_position: f64,
}

impl FiniteSound {
	fn new(mut data: Box<dyn FiniteSoundData>) -> Self {
		let buffer = (0..data.buffer_len()).map(|_| Frame::ZERO).collect();
		Self {
			data,
			buffer,
			playback_state: PlaybackState::Playing,
			current_frame_index: 0,
			fractional_playback_position: 0.0,
		}
	}

	fn update_playback_position(&mut self) {
		if self.playback_state == PlaybackState::Stopped {
			return;
		}
		self.current_frame_index += 1;
		if self.current_frame_index >= self.data.len() {
			self.playback_state = PlaybackState::Stopped;
		}
	}

	fn push_frame(&mut self) {
		let next_frame = match self.playback_state {
			PlaybackState::Playing => self.data.frame(self.current_frame_index),
			PlaybackState::Stopped => Frame::ZERO,
		};
		self.buffer.pop_front();
		self.buffer.push_back(next_frame);
		self.update_playback_position();
	}
}

impl Sound for FiniteSound {
	fn output_destination(&mut self) -> OutputDestination {
		OutputDestination::Track(TrackId::Main)
	}

	fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		self.fractional_playback_position += dt * self.data.sample_rate() as f64;
		while self.fractional_playback_position >= 1.0 {
			self.push_frame();
			self.fractional_playback_position -= 1.0;
		}
		interpolate_frame(
			self.buffer[0],
			self.buffer[1],
			self.buffer[2],
			self.buffer[3],
			self.fractional_playback_position as f32,
		)
	}

	fn finished(&self) -> bool {
		self.playback_state == PlaybackState::Stopped
			&& (0..4).all(|i| self.buffer[i] == Frame::ZERO)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PlaybackState {
	Playing,
	Stopped,
}
