/*!
Sources of audio.
*/

mod data;
mod handle;
mod settings;

pub use data::*;
pub use handle::*;
pub use settings::*;

use ringbuf::HeapConsumer;

use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::{interpolate_frame, Frame},
	modulator::value_provider::ModulatorValueProvider,
	parameter::{Parameter, Value},
	track::TrackId,
	tween::Tween,
	OutputDestination, PlaybackRate,
};

pub(crate) const COMMAND_CAPACITY: usize = 8;

pub(crate) struct Sound {
	data: Box<dyn SoundData>,
	command_consumer: HeapConsumer<Command>,
	playback_rate: Parameter<PlaybackRate>,
	current_frame_index: usize,
	fractional_playback_position: f64,
	playback_state: PlaybackState,
	resample_buffer: [Frame; 4],
}

impl Sound {
	pub fn new(
		data: Box<dyn SoundData>,
		settings: SoundSettings,
		command_consumer: HeapConsumer<Command>,
	) -> Self {
		Self {
			data,
			command_consumer,
			playback_rate: Parameter::new(settings.playback_rate, PlaybackRate::Factor(1.0)),
			current_frame_index: 0,
			fractional_playback_position: 0.0,
			playback_state: PlaybackState::Playing,
			resample_buffer: [Frame::ZERO; 4],
		}
	}

	pub fn output_destination(&mut self) -> OutputDestination {
		OutputDestination::Track(TrackId::Main)
	}

	pub fn on_start_processing(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetPlaybackRate(target, tween) => self.playback_rate.set(target, tween),
			}
		}
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		self.playback_rate
			.update(dt, clock_info_provider, modulator_value_provider);

		self.fractional_playback_position +=
			dt * self.playback_rate.value().as_factor() * self.data.sample_rate() as f64;
		while self.fractional_playback_position >= 1.0 {
			self.fractional_playback_position -= 1.0;
			self.advance_current_frame();
			self.push_to_resample_buffer();
		}
		interpolate_frame(
			self.resample_buffer[0],
			self.resample_buffer[1],
			self.resample_buffer[2],
			self.resample_buffer[3],
			self.fractional_playback_position as f32,
		)
	}

	pub fn finished(&self) -> bool {
		self.playback_state == PlaybackState::Stopped && self.resample_buffer_is_empty()
	}

	fn advance_current_frame(&mut self) {
		if self.current_frame_index >= self.data.len() {
			return;
		}
		self.current_frame_index += 1;
		if self.current_frame_index >= self.data.len() {
			self.playback_state = PlaybackState::Stopped;
		}
	}

	fn push_to_resample_buffer(&mut self) {
		for i in 0..3 {
			self.resample_buffer[i] = self.resample_buffer[i + 1];
		}
		self.resample_buffer[3] = self.data.frame(self.current_frame_index);
	}

	fn resample_buffer_is_empty(&self) -> bool {
		self.resample_buffer == [Frame::ZERO; 4]
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaybackState {
	Playing,
	Stopped,
}

pub(crate) enum Command {
	SetPlaybackRate(Value<PlaybackRate>, Tween),
}
