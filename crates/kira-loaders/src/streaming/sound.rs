pub(crate) mod decode_scheduler;

#[cfg(test)]
mod test;

use std::sync::{
	atomic::{AtomicU64, AtomicU8, Ordering},
	Arc,
};

use kira::{
	clock::ClockTime,
	dsp::{interpolate_frame, Frame},
	sound::{static_sound::PlaybackState, Sound},
	track::TrackId,
	tween::{Tween, Tweener},
	PlaybackRate, StartTime,
};
use ringbuf::Consumer;

use crate::{Command, StreamingSoundSettings};

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
	command_consumer: Consumer<Command>,
	sample_rate: u32,
	scheduler_controller: DecodeSchedulerController,
	track: TrackId,
	start_time: StartTime,
	state: PlaybackState,
	volume_fade: Tweener,
	current_frame: u64,
	fractional_position: f64,
	volume: Tweener,
	playback_rate: Tweener<PlaybackRate>,
	panning: Tweener,
	shared: Arc<Shared>,
}

impl StreamingSound {
	pub fn new<Error: Send + 'static>(
		command_consumer: Consumer<Command>,
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
			track: settings.track,
			start_time: settings.start_time,
			state: PlaybackState::Playing,
			volume_fade: if let Some(tween) = settings.fade_in_tween {
				let mut tweenable = Tweener::new(0.0);
				tweenable.set(1.0, tween);
				tweenable
			} else {
				Tweener::new(1.0)
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
		self.scheduler_controller
			.frame_consumer_mut()
			.access(|a, b| {
				let mut iter = a.iter().chain(b.iter());
				if let Some((index, _)) = iter.nth(1) {
					self.current_frame = *index;
				}
			});
	}

	fn next_frames(&mut self) -> [Frame; 4] {
		let mut frames = [Frame::ZERO; 4];
		self.scheduler_controller
			.frame_consumer_mut()
			.access(|a, b| {
				let mut iter = a.iter().chain(b.iter());
				for frame in &mut frames {
					*frame = iter
						.next()
						.copied()
						.map(|(_, frame)| frame)
						.unwrap_or(Frame::ZERO);
				}
			});
		frames
	}

	fn position(&self) -> f64 {
		(self.current_frame as f64 + self.fractional_position) / self.sample_rate as f64
	}

	fn pause(&mut self, tween: Tween) {
		self.set_state(PlaybackState::Pausing);
		self.volume_fade.set(0.0, tween);
	}

	fn resume(&mut self, tween: Tween) {
		self.set_state(PlaybackState::Playing);
		self.volume_fade.set(1.0, tween);
	}

	fn stop(&mut self, tween: Tween) {
		self.set_state(PlaybackState::Stopping);
		self.volume_fade.set(0.0, tween);
	}

	fn seek_to_index(&mut self, index: u64) {
		self.scheduler_controller.seek(index);
	}

	fn seek_to(&mut self, position: f64) {
		self.seek_to_index((position * self.sample_rate as f64).round() as u64);
	}

	fn seek_by(&mut self, amount: f64) {
		self.seek_to(self.position() + amount);
	}
}

impl Sound for StreamingSound {
	fn track(&mut self) -> TrackId {
		self.track
	}

	fn on_start_processing(&mut self) {
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

	fn process(&mut self, dt: f64) -> Frame {
		self.volume.update(dt);
		self.playback_rate.update(dt);
		self.panning.update(dt);
		if matches!(self.start_time, StartTime::ClockTime(..)) {
			return Frame::ZERO;
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
		if self.volume_fade.update(dt) {
			match self.state {
				PlaybackState::Pausing => self.set_state(PlaybackState::Paused),
				PlaybackState::Stopping => self.set_state(PlaybackState::Stopped),
				_ => {}
			}
		}
		self.update_current_frame();
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
		(out * self.volume_fade.value() as f32 * self.volume.value() as f32)
			.panned(self.panning.value() as f32)
	}

	fn on_clock_tick(&mut self, time: ClockTime) {
		self.volume.on_clock_tick(time);
		self.playback_rate.on_clock_tick(time);
		self.panning.on_clock_tick(time);
		if let StartTime::ClockTime(ClockTime { clock, ticks }) = self.start_time {
			if time.clock == clock && time.ticks >= ticks {
				self.start_time = StartTime::Immediate;
			}
		}
	}

	fn finished(&self) -> bool {
		self.state == PlaybackState::Stopped
	}
}
