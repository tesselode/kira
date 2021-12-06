use std::{
	ops::Range,
	sync::{
		atomic::{AtomicU64, AtomicU8, Ordering},
		Arc,
	},
};

use ringbuf::Consumer;

use crate::{
	clock::ClockTime,
	dsp::Frame,
	sound::Sound,
	track::TrackId,
	tween::{Tween, Tweenable},
	LoopBehavior, StartTime,
};

use super::{data::StaticSoundData, Command};

/// The playback state of a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaybackState {
	/// The sound is playing normally.
	Playing,
	/// The sound is fading out, and when the fade-out
	/// is finished, playback will pause.
	Pausing,
	/// Playback is paused.
	Paused,
	/// The sound is fading out, and when the fade-out
	/// is finished, playback will stop.
	Stopping,
	/// The sound has stopped and can no longer be resumed.
	Stopped,
}

pub(super) struct Shared {
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

pub(super) struct StaticSound {
	command_consumer: Consumer<Command>,
	data: StaticSoundData,
	start_time: StartTime,
	state: PlaybackState,
	position: f64,
	volume: f64,
	playback_rate: f64,
	panning: f64,
	volume_fade: Tweenable,
	shared: Arc<Shared>,
}

impl StaticSound {
	pub fn new(data: StaticSoundData, command_consumer: Consumer<Command>) -> Self {
		let settings = data.settings;
		let position = if settings.reverse {
			data.duration().as_secs_f64() - settings.start_position
		} else {
			settings.start_position
		};
		Self {
			command_consumer,
			data,
			start_time: settings.start_time,
			state: PlaybackState::Playing,
			position,
			volume: settings.volume,
			playback_rate: settings.playback_rate,
			panning: settings.panning,
			volume_fade: if let Some(tween) = settings.fade_in_tween {
				let mut tweenable = Tweenable::new(0.0);
				tweenable.set(1.0, tween);
				tweenable
			} else {
				Tweenable::new(1.0)
			},
			shared: Arc::new(Shared {
				state: AtomicU8::new(PlaybackState::Playing as u8),
				position: AtomicU64::new(position.to_bits()),
			}),
		}
	}

	pub(super) fn shared(&self) -> Arc<Shared> {
		self.shared.clone()
	}

	fn set_state(&mut self, state: PlaybackState) {
		self.state = state;
		self.shared.state.store(state as u8, Ordering::SeqCst);
	}

	fn pause(&mut self, fade_out_tween: Tween) {
		self.set_state(PlaybackState::Pausing);
		self.volume_fade.set(0.0, fade_out_tween);
	}

	fn resume(&mut self, fade_in_tween: Tween) {
		self.set_state(PlaybackState::Playing);
		self.volume_fade.set(1.0, fade_in_tween);
	}

	fn stop(&mut self, fade_out_tween: Tween) {
		self.set_state(PlaybackState::Stopping);
		self.volume_fade.set(0.0, fade_out_tween);
	}

	fn playback_rate(&self) -> f64 {
		if self.data.settings.reverse {
			-self.playback_rate
		} else {
			self.playback_rate
		}
	}

	fn increment_playback_position(&mut self, amount: f64) {
		self.position += amount;
		if let Some(LoopBehavior { start_position }) = self.data.settings.loop_behavior {
			self.position = wrap(
				self.position,
				start_position..self.data.duration().as_secs_f64(),
			);
		} else if self.position < 0.0 || self.position > self.data.duration().as_secs_f64() {
			self.set_state(PlaybackState::Stopped);
		}
	}
}

impl Sound for StaticSound {
	fn track(&mut self) -> TrackId {
		self.data.settings.track
	}

	fn on_start_processing(&mut self) {
		self.shared
			.position
			.store(self.position.to_bits(), Ordering::SeqCst);
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetVolume(volume) => self.volume = volume,
				Command::SetPlaybackRate(playback_rate) => self.playback_rate = playback_rate,
				Command::SetPanning(panning) => self.panning = panning,
				Command::Pause(tween) => self.pause(tween),
				Command::Resume(tween) => self.resume(tween),
				Command::Stop(tween) => self.stop(tween),
				Command::SeekBy(amount) => self.increment_playback_position(amount),
				Command::SeekTo(position) => {
					self.increment_playback_position(position - self.position)
				}
			}
		}
	}

	fn process(&mut self, dt: f64) -> Frame {
		if matches!(self.start_time, StartTime::ClockTime(..)) {
			return Frame::ZERO;
		}
		if self.volume_fade.update(dt) {
			match self.state {
				PlaybackState::Pausing => self.set_state(PlaybackState::Paused),
				PlaybackState::Stopping => self.set_state(PlaybackState::Stopped),
				_ => {}
			}
		}
		if matches!(self.state, PlaybackState::Paused | PlaybackState::Stopped) {
			return Frame::ZERO;
		}
		let out = self.data.frame_at_position(self.position);
		self.increment_playback_position(self.playback_rate() * dt);
		(out * self.volume_fade.value() as f32 * self.volume as f32).panned(self.panning as f32)
	}

	fn on_clock_tick(&mut self, time: ClockTime) {
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

fn wrap(mut x: f64, range: Range<f64>) -> f64 {
	let length = range.end - range.start;
	while x < range.start {
		x += length;
	}
	while x > range.end {
		x -= length;
	}
	x
}
