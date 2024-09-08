pub(crate) mod decode_scheduler;

#[cfg(test)]
mod test;

use std::sync::{
	atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
	Arc,
};

use crate::{
	command::read_commands_into_parameters,
	frame::{interpolate_frame, Frame},
	info::Info,
	playback_state_manager::PlaybackStateManager,
	sound::{PlaybackRate, PlaybackState, Sound},
	tween::{Parameter, Tween},
	Dbfs, StartTime,
};
use ringbuf::HeapConsumer;

use super::{CommandReaders, StreamingSoundSettings};

use self::decode_scheduler::DecodeScheduler;

#[derive(Debug)]
pub(crate) struct Shared {
	state: AtomicU8,
	position: AtomicU64,
	reached_end: AtomicBool,
	encountered_error: AtomicBool,
}

impl Shared {
	#[must_use]
	pub fn new() -> Self {
		Self {
			position: AtomicU64::new(0.0f64.to_bits()),
			state: AtomicU8::new(PlaybackState::Playing as u8),
			reached_end: AtomicBool::new(false),
			encountered_error: AtomicBool::new(false),
		}
	}

	#[must_use]
	pub fn state(&self) -> PlaybackState {
		match self.state.load(Ordering::SeqCst) {
			0 => PlaybackState::Playing,
			1 => PlaybackState::Pausing,
			2 => PlaybackState::Paused,
			3 => PlaybackState::WaitingToResume,
			4 => PlaybackState::Resuming,
			5 => PlaybackState::Stopping,
			6 => PlaybackState::Stopped,
			_ => panic!("Invalid playback state"),
		}
	}

	pub fn set_state(&self, state: PlaybackState) {
		self.state.store(state as u8, Ordering::SeqCst);
	}

	#[must_use]
	pub fn position(&self) -> f64 {
		f64::from_bits(self.position.load(Ordering::SeqCst))
	}

	#[must_use]
	pub fn reached_end(&self) -> bool {
		self.reached_end.load(Ordering::SeqCst)
	}

	#[must_use]
	pub fn encountered_error(&self) -> bool {
		self.encountered_error.load(Ordering::SeqCst)
	}
}

pub(crate) struct StreamingSound {
	command_readers: CommandReaders,
	sample_rate: u32,
	frame_consumer: HeapConsumer<TimestampedFrame>,
	start_time: StartTime,
	playback_state_manager: PlaybackStateManager,
	current_frame: usize,
	fractional_position: f64,
	volume: Parameter<Dbfs>,
	playback_rate: Parameter<PlaybackRate>,
	panning: Parameter,
	shared: Arc<Shared>,
}

impl StreamingSound {
	#[must_use]
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
			start_time: settings.start_time,
			playback_state_manager: PlaybackStateManager::new(settings.fade_in_tween),
			current_frame,
			fractional_position: 0.0,
			volume: Parameter::new(settings.volume, Dbfs::MAX),
			playback_rate: Parameter::new(settings.playback_rate, PlaybackRate::Factor(1.0)),
			panning: Parameter::new(settings.panning, 0.5),
			shared,
		}
	}

	fn update_shared_playback_state(&mut self) {
		self.shared
			.set_state(self.playback_state_manager.playback_state());
	}

	fn update_current_frame(&mut self) {
		let current_frame = &mut self.current_frame;
		let (a, b) = self.frame_consumer.as_slices();
		let mut iter = a.iter().chain(b.iter());
		if let Some(TimestampedFrame { index, .. }) = iter.nth(1) {
			*current_frame = *index;
		}
	}

	#[must_use]
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

	#[must_use]
	fn position(&self) -> f64 {
		(self.current_frame as f64 + self.fractional_position) / self.sample_rate as f64
	}

	fn pause(&mut self, fade_out_tween: Tween) {
		self.playback_state_manager.pause(fade_out_tween);
		self.update_shared_playback_state();
	}

	fn resume(&mut self, start_time: StartTime, fade_in_tween: Tween) {
		self.playback_state_manager
			.resume(start_time, fade_in_tween);
		self.update_shared_playback_state();
	}

	fn stop(&mut self, fade_out_tween: Tween) {
		self.playback_state_manager.stop(fade_out_tween);
		self.update_shared_playback_state();
	}

	fn read_commands(&mut self) {
		read_commands_into_parameters!(self, volume, playback_rate, panning);
		if let Some(tween) = self.command_readers.pause.read() {
			self.pause(tween);
		}
		if let Some((start_time, tween)) = self.command_readers.resume.read() {
			self.resume(start_time, tween);
		}
		if let Some(tween) = self.command_readers.stop.read() {
			self.stop(tween);
		}
	}
}

impl Sound for StreamingSound {
	fn on_start_processing(&mut self) {
		self.update_current_frame();
		self.shared
			.position
			.store(self.position().to_bits(), Ordering::SeqCst);
		self.read_commands();
	}

	fn process(&mut self, dt: f64, info: &Info) -> Frame {
		if self.shared.encountered_error() {
			self.playback_state_manager.mark_as_stopped();
			self.update_shared_playback_state();
			return Frame::ZERO;
		}

		// update parameters
		self.volume.update(dt, info);
		self.playback_rate.update(dt, info);
		self.panning.update(dt, info);
		let changed_playback_state = self.playback_state_manager.update(dt, info);
		if changed_playback_state {
			self.update_shared_playback_state();
		}

		let will_never_start = self.start_time.update(dt, info);
		if will_never_start {
			self.playback_state_manager.mark_as_stopped();
			self.update_shared_playback_state();
		}
		if self.start_time != StartTime::Immediate {
			return Frame::ZERO;
		}

		if !self.playback_state_manager.playback_state().is_advancing() {
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
			self.playback_state_manager.mark_as_stopped();
			self.update_shared_playback_state();
		}
		(out * self.playback_state_manager.fade_volume().as_amplitude()
			* self.volume.value().as_amplitude())
			.panned(self.panning.value() as f32)
	}

	fn finished(&self) -> bool {
		self.playback_state_manager.playback_state() == PlaybackState::Stopped
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TimestampedFrame {
	frame: Frame,
	index: usize,
}
