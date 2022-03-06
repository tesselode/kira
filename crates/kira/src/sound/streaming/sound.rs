pub(crate) mod decode_scheduler;

#[cfg(test)]
mod test;

use std::sync::{
	atomic::{AtomicU64, AtomicU8, Ordering},
	Arc,
};

use crate::{
	clock::clock_info::{ClockInfoProvider, WhenToStart},
	dsp::{interpolate_frame, Frame},
	sound::{static_sound::PlaybackState, Sound},
	tween::{Tween, Tweener},
	OutputDestination, PlaybackRate, StartTime, Volume,
};
use ringbuf::HeapConsumer;

use super::{Command, StreamingSoundSettings};

use self::decode_scheduler::{DecodeScheduler, DecodeSchedulerController};

pub(crate) struct Shared {
	state: AtomicU8,
	position: AtomicU64,
}

impl Shared {
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
}

pub(crate) struct StreamingSound {
	command_consumer: HeapConsumer<Command>,
	sample_rate: u32,
	scheduler_controller: DecodeSchedulerController,
	output_destination: OutputDestination,
	start_time: StartTime,
	state: PlaybackState,
	when_to_start: WhenToStart,
	volume_fade: Tweener<Volume>,
	current_frame: i64,
	fractional_position: f64,
	volume: Tweener<Volume>,
	playback_rate: Tweener<PlaybackRate>,
	panning: Tweener,
	shared: Arc<Shared>,
}

impl StreamingSound {
	pub fn new<Error: Send + 'static>(
		command_consumer: HeapConsumer<Command>,
		scheduler_controller: DecodeSchedulerController,
		settings: StreamingSoundSettings,
		sample_rate: u32,
		scheduler: &DecodeScheduler<Error>,
	) -> Self {
		let current_frame = scheduler.current_frame();
		let start_position = current_frame as f64 / sample_rate as f64;
		Self {
			command_consumer,
			sample_rate,
			scheduler_controller,
			output_destination: settings.output_destination,
			start_time: settings.start_time,
			state: PlaybackState::Playing,
			when_to_start: if matches!(settings.start_time, StartTime::ClockTime(..)) {
				WhenToStart::Later
			} else {
				WhenToStart::Now
			},
			volume_fade: if let Some(tween) = settings.fade_in_tween {
				let mut tweenable = Tweener::new(Volume::Decibels(Volume::MIN_DECIBELS));
				tweenable.set(Volume::Decibels(0.0), tween);
				tweenable
			} else {
				Tweener::new(Volume::Decibels(0.0))
			},
			current_frame,
			fractional_position: 0.0,
			volume: Tweener::new(settings.volume),
			playback_rate: Tweener::new(settings.playback_rate),
			panning: Tweener::new(settings.panning),
			shared: Arc::new(Shared {
				position: AtomicU64::new(start_position.to_bits()),
				state: AtomicU8::new(PlaybackState::Playing as u8),
			}),
		}
	}

	pub fn shared(&self) -> Arc<Shared> {
		self.shared.clone()
	}

	fn set_state(&mut self, state: PlaybackState) {
		self.state = state;
		self.shared.state.store(state as u8, Ordering::SeqCst);
	}

	fn update_current_frame(&mut self) {
		let current_frame = &mut self.current_frame;
		let (a, b) = self.scheduler_controller.frame_consumer_mut().as_slices();
		let mut iter = a.iter().chain(b.iter());
		if let Some((index, _)) = iter.nth(1) {
			*current_frame = *index;
		}
	}

	fn next_frames(&mut self) -> [Frame; 4] {
		let mut frames = [Frame::ZERO; 4];
		let (a, b) = self.scheduler_controller.frame_consumer_mut().as_slices();
		let mut iter = a.iter().chain(b.iter());
		for frame in &mut frames {
			*frame = iter
				.next()
				.copied()
				.map(|(_, frame)| frame)
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
			.set(Volume::Decibels(Volume::MIN_DECIBELS), tween);
	}

	fn resume(&mut self, tween: Tween) {
		self.set_state(PlaybackState::Playing);
		self.volume_fade.set(Volume::Decibels(0.0), tween);
	}

	fn stop(&mut self, tween: Tween) {
		self.set_state(PlaybackState::Stopping);
		self.volume_fade
			.set(Volume::Decibels(Volume::MIN_DECIBELS), tween);
	}

	fn seek_to_index(&mut self, index: i64) {
		self.scheduler_controller.seek(index);
	}

	fn seek_to(&mut self, position: f64) {
		self.seek_to_index((position * self.sample_rate as f64).round() as i64);
	}

	fn seek_by(&mut self, amount: f64) {
		self.seek_to(self.position() + amount);
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
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetVolume(volume, tween) => self.volume.set(volume, tween),
				Command::SetPlaybackRate(playback_rate, tween) => {
					self.playback_rate.set(playback_rate, tween)
				}
				Command::SetPanning(panning, tween) => self.panning.set(panning, tween),
				Command::Pause(tween) => self.pause(tween),
				Command::Resume(tween) => self.resume(tween),
				Command::Stop(tween) => self.stop(tween),
				Command::SeekBy(amount) => self.seek_by(amount),
				Command::SeekTo(position) => self.seek_to(position),
			}
		}
	}

	fn process(&mut self, dt: f64, clock_info_provider: &ClockInfoProvider) -> Frame {
		// update tweeners
		self.volume.update(dt, clock_info_provider);
		self.playback_rate.update(dt, clock_info_provider);
		self.panning.update(dt, clock_info_provider);
		if self.volume_fade.update(dt, clock_info_provider) {
			match self.state {
				PlaybackState::Pausing => self.set_state(PlaybackState::Paused),
				PlaybackState::Stopping => self.set_state(PlaybackState::Stopped),
				_ => {}
			}
		}

		// for sounds waiting on a clock, check if it's ready to start
		match self.when_to_start {
			WhenToStart::Now => {}
			// if the sound is waiting for a start time, check the clock info
			// provider for a change in that status
			WhenToStart::Later => {
				self.when_to_start = clock_info_provider.when_to_start(self.start_time);
				match self.when_to_start {
					WhenToStart::Now => {}
					// if the sound is still waiting, return silence
					WhenToStart::Later => return Frame::ZERO,
					// if we learn that the sound will never start,
					// stop the sound and return silence
					WhenToStart::Never => {
						self.stop(Tween::default());
						return Frame::ZERO;
					}
				}
			}
			// if we already know the sound will never start, output silence
			WhenToStart::Never => return Frame::ZERO,
		}

		if matches!(self.state, PlaybackState::Paused | PlaybackState::Stopped) {
			return Frame::ZERO;
		}
		// pause playback while waiting for audio data. the first frame
		// in the ringbuffer is the previous frame, so we need to make
		// sure there's at least 2 before we continue playing.
		if self.scheduler_controller.frame_consumer_mut().len() < 2
			&& !self.scheduler_controller.finished()
		{
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
			self.sample_rate as f64 * self.playback_rate.value().as_factor() * dt;
		while self.fractional_position >= 1.0 {
			self.fractional_position -= 1.0;
			self.scheduler_controller.frame_consumer_mut().pop();
		}
		if self.scheduler_controller.finished()
			&& self.scheduler_controller.frame_consumer_mut().is_empty()
		{
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
