mod error;
mod handle;
mod settings;
mod static_sound;
#[cfg(feature = "symphonia")]
mod symphonia;

use std::collections::VecDeque;

pub use error::*;
pub use handle::*;
use ringbuf::HeapConsumer;
pub use settings::*;
pub use static_sound::*;

use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::{interpolate_frame, Frame},
	modulator::value_provider::ModulatorValueProvider,
	parameter::{Parameter, Value},
	track::TrackId,
	tween::Tween,
	OutputDestination, PlaybackRate, Volume,
};

use super::Sound;

const COMMAND_BUFFER_CAPACITY: usize = 8;

trait FiniteSoundData: Send {
	fn sample_rate(&mut self) -> u32;

	fn len(&mut self) -> usize;

	fn frame(&mut self, index: usize) -> Frame;

	fn buffer_len(&mut self) -> usize;
}

struct FiniteSound {
	data: Box<dyn FiniteSoundData>,
	command_consumer: HeapConsumer<Command>,
	volume: Parameter<Volume>,
	playback_rate: Parameter<PlaybackRate>,
	panning: Parameter<f64>,
	buffer: VecDeque<Frame>,
	playback_state: PlaybackState,
	current_frame_index: usize,
	fractional_playback_position: f64,
}

impl FiniteSound {
	fn new(
		mut data: Box<dyn FiniteSoundData>,
		settings: SoundSettings,
		command_consumer: HeapConsumer<Command>,
	) -> Self {
		let buffer = (0..data.buffer_len()).map(|_| Frame::ZERO).collect();
		Self {
			data,
			command_consumer,
			volume: Parameter::new(settings.volume, Volume::Amplitude(1.0)),
			playback_rate: Parameter::new(settings.playback_rate, PlaybackRate::Factor(1.0)),
			panning: Parameter::new(settings.panning, 0.5),
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

	fn update_parameters(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) {
		self.volume
			.update(dt, clock_info_provider, modulator_value_provider);
		self.playback_rate
			.update(dt, clock_info_provider, modulator_value_provider);
		self.panning
			.update(dt, clock_info_provider, modulator_value_provider);
	}
}

impl Sound for FiniteSound {
	fn output_destination(&mut self) -> OutputDestination {
		OutputDestination::Track(TrackId::Main)
	}

	fn on_start_processing(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetVolume(target, tween) => self.volume.set(target, tween),
				Command::SetPlaybackRate(target, tween) => self.playback_rate.set(target, tween),
				Command::SetPanning(target, tween) => self.panning.set(target, tween),
				Command::Pause(_) => todo!(),
				Command::Resume(_) => todo!(),
				Command::Stop(_) => todo!(),
				Command::SeekBy(_) => todo!(),
				Command::SeekTo(_) => todo!(),
			}
		}
	}

	fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		self.update_parameters(dt, clock_info_provider, modulator_value_provider);
		self.fractional_playback_position +=
			self.playback_rate.value().as_factor() * dt * self.data.sample_rate() as f64;
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
		.panned(self.panning.value() as f32)
			* self.volume.value().as_amplitude() as f32
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum Command {
	SetVolume(Value<Volume>, Tween),
	SetPlaybackRate(Value<PlaybackRate>, Tween),
	SetPanning(Value<f64>, Tween),
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
	SeekBy(f64),
	SeekTo(f64),
}
