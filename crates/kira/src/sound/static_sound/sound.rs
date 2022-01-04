mod resampler;

use std::sync::{
	atomic::{AtomicU64, AtomicU8, Ordering},
	Arc,
};

use ringbuf::Consumer;

use crate::{
	clock::ClockTime,
	dsp::Frame,
	sound::Sound,
	track::TrackId,
	tween::{Tween, Tweener},
	LoopBehavior, PlaybackRate, StartTime, Volume,
};

use self::resampler::Resampler;

use super::{data::StaticSoundData, Command};

#[cfg(test)]
mod test;

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
	resampler: Resampler,
	current_sample_index: usize,
	fractional_position: f64,
	volume: Tweener<Volume>,
	playback_rate: Tweener<PlaybackRate>,
	panning: Tweener,
	volume_fade: Tweener<Volume>,
	shared: Arc<Shared>,
}

impl StaticSound {
	pub fn new(data: StaticSoundData, command_consumer: Consumer<Command>) -> Self {
		let settings = data.settings;
		let current_sample_index = if settings.reverse {
			let position_seconds = data.duration().as_secs_f64() - settings.start_position;
			(position_seconds * data.sample_rate as f64) as usize - 1
		} else {
			(settings.start_position * data.sample_rate as f64) as usize
		};
		let position = current_sample_index as f64 / data.sample_rate as f64;
		let mut sound = Self {
			command_consumer,
			data,
			start_time: settings.start_time,
			state: PlaybackState::Playing,
			resampler: Resampler::new(),
			current_sample_index,
			fractional_position: 0.0,
			volume: Tweener::new(settings.volume),
			playback_rate: Tweener::new(settings.playback_rate),
			panning: Tweener::new(settings.panning),
			volume_fade: if let Some(tween) = settings.fade_in_tween {
				let mut tweenable = Tweener::new(Volume::Decibels(Volume::MIN_DECIBELS));
				tweenable.set(Volume::Decibels(0.0), tween);
				tweenable
			} else {
				Tweener::new(Volume::Decibels(0.0))
			},
			shared: Arc::new(Shared {
				state: AtomicU8::new(PlaybackState::Playing as u8),
				position: AtomicU64::new(position.to_bits()),
			}),
		};
		// fill the resample buffer with 3 samples so playback can
		// start immediately
		for _ in 0..3 {
			sound.update_position();
		}
		sound
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
		self.volume_fade
			.set(Volume::Decibels(Volume::MIN_DECIBELS), fade_out_tween);
	}

	fn resume(&mut self, fade_in_tween: Tween) {
		self.set_state(PlaybackState::Playing);
		self.volume_fade.set(Volume::Decibels(0.0), fade_in_tween);
	}

	fn stop(&mut self, fade_out_tween: Tween) {
		self.set_state(PlaybackState::Stopping);
		self.volume_fade
			.set(Volume::Decibels(Volume::MIN_DECIBELS), fade_out_tween);
	}

	fn playback_rate(&self) -> f64 {
		if self.data.settings.reverse {
			-self.playback_rate.value().as_factor()
		} else {
			self.playback_rate.value().as_factor()
		}
	}

	/// Increments the playback position by 1 sample. Returns `true` if the end
	/// of the sound was reached.
	fn increment_position(&mut self) -> bool {
		if let Some(LoopBehavior { start_position }) = self.data.settings.loop_behavior {
			let start_position = (start_position * self.data.sample_rate as f64) as usize;
			if self.current_sample_index >= self.data.frames.len() - 1 {
				self.current_sample_index = start_position;
			} else {
				self.current_sample_index += 1;
			}
		} else {
			if self.current_sample_index >= self.data.frames.len() - 1 {
				return true;
			} else {
				self.current_sample_index += 1;
			}
		}
		false
	}

	/// Decrements the playback position by 1 sample. Returns `true` if the end
	/// of the sound was reached (which in this case would be sample -1).
	fn decrement_position(&mut self) -> bool {
		if let Some(LoopBehavior { start_position }) = self.data.settings.loop_behavior {
			let start_position = (start_position * self.data.sample_rate as f64) as usize;
			if self.current_sample_index <= start_position {
				self.current_sample_index = self.data.frames.len() - 1;
			} else {
				self.current_sample_index -= 1;
			}
		} else {
			if self.current_sample_index == 0 {
				return true;
			} else {
				self.current_sample_index -= 1;
			}
		}
		false
	}

	/// Updates the playback position and pushes a new sample to the resampler.
	fn update_position(&mut self) {
		let playback_rate = self.playback_rate();
		if matches!(self.state, PlaybackState::Paused | PlaybackState::Stopped) {
			self.resampler.push_frame(Frame::ZERO, None);
			return;
		}
		let out = self.data.frames[self.current_sample_index];
		let out = (out
			* self.volume_fade.value().as_amplitude() as f32
			* self.volume.value().as_amplitude() as f32)
			.panned(self.panning.value() as f32);
		self.resampler.push_frame(out, self.current_sample_index);
		let reached_end_of_sound = if playback_rate.is_sign_negative() {
			self.decrement_position()
		} else {
			self.increment_position()
		};
		if reached_end_of_sound {
			self.set_state(PlaybackState::Stopped);
		}
	}

	fn seek_to_index(&mut self, index: usize) {
		self.current_sample_index = index;
		// if the seek index is past the end of the sound and the sound is
		// looping, wrap the seek point back into the sound
		if let Some(LoopBehavior { start_position }) = self.data.settings.loop_behavior {
			let start_position = (start_position * self.data.sample_rate as f64) as usize;
			while self.current_sample_index >= self.data.frames.len() {
				self.current_sample_index -= self.data.frames.len() - start_position;
			}
		// otherwise, stop the sound
		} else if self.current_sample_index >= self.data.frames.len() {
			self.set_state(PlaybackState::Stopped);
		}
		// if the sound is playing, push a frame to the resample buffer
		// to make sure it doesn't get skipped
		if matches!(self.state, PlaybackState::Paused | PlaybackState::Stopped) {
			return;
		}
		let out = self.data.frames[self.current_sample_index];
		let out = (out
			* self.volume_fade.value().as_amplitude() as f32
			* self.volume.value().as_amplitude() as f32)
			.panned(self.panning.value() as f32);
		self.resampler.push_frame(out, self.current_sample_index);
	}
}

impl Sound for StaticSound {
	fn track(&mut self) -> TrackId {
		self.data.settings.track
	}

	fn on_start_processing(&mut self) {
		let last_played_frame_position = self
			.resampler
			.position()
			.expect("The resampler has not received any frames yet");
		self.shared.position.store(
			(last_played_frame_position as f64 / self.data.sample_rate as f64).to_bits(),
			Ordering::SeqCst,
		);
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
				Command::SeekBy(amount) => {
					let current_position =
						self.current_sample_index as f64 / self.data.sample_rate as f64;
					let position = current_position + amount;
					let index = (position * self.data.sample_rate as f64) as usize;
					self.seek_to_index(index);
				}
				Command::SeekTo(position) => {
					let index = (position * self.data.sample_rate as f64) as usize;
					self.seek_to_index(index);
				}
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
		if self.volume_fade.update(dt) {
			match self.state {
				PlaybackState::Pausing => self.set_state(PlaybackState::Paused),
				PlaybackState::Stopping => self.set_state(PlaybackState::Stopped),
				_ => {}
			}
		}
		let out = self.resampler.get(self.fractional_position as f32);
		self.fractional_position += self.data.sample_rate as f64 * self.playback_rate().abs() * dt;
		while self.fractional_position >= 1.0 {
			self.fractional_position -= 1.0;
			self.update_position();
		}
		out
	}

	fn on_clock_tick(&mut self, time: ClockTime) {
		self.volume.on_clock_tick(time);
		self.playback_rate.on_clock_tick(time);
		self.panning.on_clock_tick(time);
		self.volume_fade.on_clock_tick(time);
		if let StartTime::ClockTime(ClockTime { clock, ticks }) = self.start_time {
			if time.clock == clock && time.ticks >= ticks {
				self.start_time = StartTime::Immediate;
			}
		}
	}

	fn finished(&self) -> bool {
		self.state == PlaybackState::Stopped && self.resampler.is_empty()
	}
}
