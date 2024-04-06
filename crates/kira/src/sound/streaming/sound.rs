pub(crate) mod decode_scheduler;

#[cfg(test)]
mod test;

use std::{
	sync::{
		atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
		Arc,
	},
	time::Duration,
};

use crate::{
	clock::clock_info::{ClockInfoProvider, WhenToStart},
	dsp::{interpolate_frame, Frame},
	modulator::value_provider::ModulatorValueProvider,
	read_commands_into_parameters,
	sound::{util::create_volume_fade_parameter, PlaybackRate, PlaybackState, Sound},
	tween::{Parameter, Tween, Value},
	OutputDestination, StartTime, Volume,
};
use ringbuf::HeapConsumer;

use super::{CommandReaders, StreamingSoundSettings};

use self::decode_scheduler::DecodeScheduler;

pub(crate) struct Shared {
	state: AtomicU8,
	position: AtomicU64,
	reached_end: AtomicBool,
}

impl Shared {
	pub fn new() -> Self {
		Self {
			position: AtomicU64::new(0.0f64.to_bits()),
			state: AtomicU8::new(PlaybackState::Playing as u8),
			reached_end: AtomicBool::new(false),
		}
	}

	pub fn state(&self) -> PlaybackState {
		match self.state.load(Ordering::SeqCst) {
			0 => PlaybackState::Playing,
			1 => PlaybackState::Pausing,
			2 => PlaybackState::Paused,
			3 => PlaybackState::Stopping,
			4 => PlaybackState::Stopped,
			_ => panic!("Invalid playback state"),
		}
	}

	pub fn position(&self) -> f64 {
		f64::from_bits(self.position.load(Ordering::SeqCst))
	}

	pub fn reached_end(&self) -> bool {
		self.reached_end.load(Ordering::SeqCst)
	}
}

pub(crate) struct StreamingSound {
	command_readers: CommandReaders,
	sample_rate: u32,
	frame_consumer: HeapConsumer<TimestampedFrame>,
	output_destination: OutputDestination,
	start_time: StartTime,
	state: PlaybackState,
	volume_fade: Parameter<Volume>,
	current_frame: usize,
	fractional_position: f64,
	volume: Parameter<Volume>,
	playback_rate: Parameter<PlaybackRate>,
	panning: Parameter,
	shared: Arc<Shared>,
}

impl StreamingSound {
	pub(super) fn new<Error: Send + 'static>(
		sample_rate: u32,
		settings: StreamingSoundSettings,
		shared: Arc<Shared>,
		frame_consumer: HeapConsumer<TimestampedFrame>,
		command_readers: CommandReaders,
		scheduler: &DecodeScheduler<Error>,
	) -> Self {
		let current_frame = scheduler.current_frame();
		let start_position = current_frame as f64 / sample_rate as f64;
		shared
			.position
			.store(start_position.to_bits(), Ordering::SeqCst);
		Self {
			command_readers,
			sample_rate,
			frame_consumer,
			output_destination: settings.output_destination,
			start_time: settings.start_time,
			state: PlaybackState::Playing,
			volume_fade: create_volume_fade_parameter(settings.fade_in_tween),
			current_frame,
			fractional_position: 0.0,
			volume: Parameter::new(settings.volume, Volume::Amplitude(1.0)),
			playback_rate: Parameter::new(settings.playback_rate, PlaybackRate::Factor(1.0)),
			panning: Parameter::new(settings.panning, 0.5),
			shared,
		}
	}

	fn set_state(&mut self, state: PlaybackState) {
		self.state = state;
		self.shared.state.store(state as u8, Ordering::SeqCst);
	}

	fn update_current_frame(&mut self) {
		let current_frame = &mut self.current_frame;
		let (a, b) = self.frame_consumer.as_slices();
		let mut iter = a.iter().chain(b.iter());
		if let Some(TimestampedFrame { index, .. }) = iter.nth(1) {
			*current_frame = *index;
		}
	}

	fn next_frames(&mut self) -> [Frame; 4] {
		let mut frames = [Frame::ZERO; 4];
		let (a, b) = self.frame_consumer.as_slices();
		let mut iter = a.iter().chain(b.iter());
		for frame in &mut frames {
			*frame = iter
				.next()
				.copied()
				.map(|TimestampedFrame { frame, .. }| frame)
				.unwrap_or(Frame::ZERO);
		}
		frames
	}

	fn position(&self) -> f64 {
		(self.current_frame as f64 + self.fractional_position) / self.sample_rate as f64
	}

	fn pause(&mut self, tween: Tween) {
		self.set_state(PlaybackState::Pausing);
		self.volume_fade
			.set(Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)), tween);
	}

	fn resume(&mut self, tween: Tween) {
		self.set_state(PlaybackState::Playing);
		self.volume_fade
			.set(Value::Fixed(Volume::Decibels(0.0)), tween);
	}

	fn stop(&mut self, tween: Tween) {
		self.set_state(PlaybackState::Stopping);
		self.volume_fade
			.set(Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)), tween);
	}

	fn read_commands(&mut self) {
		read_commands_into_parameters!(self, volume, playback_rate, panning);
		if let Some(tween) = self.command_readers.pause.read() {
			self.pause(tween);
		}
		if let Some(tween) = self.command_readers.resume.read() {
			self.resume(tween);
		}
		if let Some(tween) = self.command_readers.stop.read() {
			self.stop(tween);
		}
	}
}

impl Sound for StreamingSound {
	fn output_destination(&mut self) -> OutputDestination {
		self.output_destination
	}

	fn on_start_processing(&mut self) {
		self.update_current_frame();
		self.shared
			.position
			.store(self.position().to_bits(), Ordering::SeqCst);
		self.read_commands();
	}

	fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		// update parameters
		self.volume
			.update(dt, clock_info_provider, modulator_value_provider);
		self.playback_rate
			.update(dt, clock_info_provider, modulator_value_provider);
		self.panning
			.update(dt, clock_info_provider, modulator_value_provider);
		if self
			.volume_fade
			.update(dt, clock_info_provider, modulator_value_provider)
		{
			match self.state {
				PlaybackState::Pausing => self.set_state(PlaybackState::Paused),
				PlaybackState::Stopping => self.set_state(PlaybackState::Stopped),
				_ => {}
			}
		}

		// check if the sound has started
		let started = match &mut self.start_time {
			StartTime::Immediate => true,
			StartTime::Delayed(time_remaining) => {
				if time_remaining.is_zero() {
					true
				} else {
					*time_remaining = time_remaining.saturating_sub(Duration::from_secs_f64(dt));
					false
				}
			}
			StartTime::ClockTime(clock_time) => {
				match clock_info_provider.when_to_start(*clock_time) {
					WhenToStart::Now => true,
					WhenToStart::Later => false,
					WhenToStart::Never => {
						self.set_state(PlaybackState::Stopped);
						false
					}
				}
			}
		};
		if !started {
			return Frame::ZERO;
		}

		if matches!(self.state, PlaybackState::Paused | PlaybackState::Stopped) {
			return Frame::ZERO;
		}
		// pause playback while waiting for audio data. the first frame
		// in the ringbuffer is the previous frame, so we need to make
		// sure there's at least 2 before we continue playing.
		if self.frame_consumer.len() < 2 && !self.shared.reached_end() {
			return Frame::ZERO;
		}
		let next_frames = self.next_frames();
		let out = interpolate_frame(
			next_frames[0],
			next_frames[1],
			next_frames[2],
			next_frames[3],
			self.fractional_position as f32,
		);
		self.fractional_position +=
			self.sample_rate as f64 * self.playback_rate.value().as_factor().max(0.0) * dt;
		while self.fractional_position >= 1.0 {
			self.fractional_position -= 1.0;
			self.frame_consumer.pop();
		}
		if self.shared.reached_end() && self.frame_consumer.is_empty() {
			self.set_state(PlaybackState::Stopped);
		}
		(out * self.volume_fade.value().as_amplitude() as f32
			* self.volume.value().as_amplitude() as f32)
			.panned(self.panning.value() as f32)
	}

	fn finished(&self) -> bool {
		self.state == PlaybackState::Stopped
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TimestampedFrame {
	frame: Frame,
	index: usize,
}
