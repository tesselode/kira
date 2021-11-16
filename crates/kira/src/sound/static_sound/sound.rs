use std::{
	ops::Range,
	sync::{
		atomic::{AtomicU64, AtomicU8, Ordering},
		Arc,
	},
};

use ringbuf::Consumer;

use crate::{
	clock::{ClockTime, Clocks},
	dsp::Frame,
	parameter::Parameters,
	sound::Sound,
	track::TrackId,
	tween::{Tween, Tweenable},
	value::CachedValue,
	LoopBehavior, StartTime,
};

use super::{data::StaticSoundData, Command};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaybackState {
	Playing,
	Pausing,
	Paused,
	Stopping,
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
	volume: CachedValue,
	playback_rate: CachedValue,
	panning: CachedValue,
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
			volume: CachedValue::new(.., settings.volume, 1.0),
			playback_rate: CachedValue::new(.., settings.playback_rate, 1.0),
			panning: CachedValue::new(0.0..=1.0, settings.panning, 0.5),
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
			-self.playback_rate.get()
		} else {
			self.playback_rate.get()
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
				Command::SetVolume(volume) => self.volume.set(volume),
				Command::SetPlaybackRate(playback_rate) => self.playback_rate.set(playback_rate),
				Command::SetPanning(panning) => self.panning.set(panning),
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

	fn process(&mut self, dt: f64, parameters: &Parameters, clocks: &Clocks) -> Frame {
		if let StartTime::ClockTime(ClockTime { clock, ticks }) = self.start_time {
			if let Some(clock) = clocks.get(clock) {
				if clock.ticking() && clock.ticks() >= ticks {
					self.start_time = StartTime::Immediate;
				}
			}
		}
		if matches!(self.start_time, StartTime::ClockTime(..)) {
			return Frame::ZERO;
		}
		if self.volume_fade.update(dt, clocks) {
			match self.state {
				PlaybackState::Pausing => self.set_state(PlaybackState::Paused),
				PlaybackState::Stopping => self.set_state(PlaybackState::Stopped),
				_ => {}
			}
		}
		if matches!(self.state, PlaybackState::Paused | PlaybackState::Stopped) {
			return Frame::ZERO;
		}
		self.volume.update(parameters);
		self.playback_rate.update(parameters);
		self.panning.update(parameters);
		let out = self.data.frame_at_position(self.position);
		self.increment_playback_position(self.playback_rate() * dt);
		(out * self.volume_fade.value() as f32 * self.volume.get() as f32)
			.panned(self.panning.get() as f32)
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
