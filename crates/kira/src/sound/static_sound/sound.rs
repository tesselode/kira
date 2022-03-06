mod resampler;

#[cfg(test)]
mod test;

use std::{
	convert::TryInto,
	sync::{
		atomic::{AtomicU64, AtomicU8, Ordering},
		Arc,
	},
};

use ringbuf::HeapConsumer;

use crate::{
	clock::clock_info::{ClockInfoProvider, WhenToStart},
	dsp::Frame,
	sound::Sound,
	tween::{Tween, Tweener},
	LoopBehavior, OutputDestination, PlaybackRate, StartTime, Volume,
};

use self::resampler::Resampler;

use super::{data::StaticSoundData, Command, StaticSoundSettings};

pub(super) struct StaticSound {
	command_consumer: HeapConsumer<Command>,
	data: StaticSoundData,
	state: PlaybackState,
	when_to_start: WhenToStart,
	resampler: Resampler,
	current_frame_index: i64,
	fractional_position: f64,
	volume: Tweener<Volume>,
	playback_rate: Tweener<PlaybackRate>,
	panning: Tweener,
	volume_fade: Tweener<Volume>,
	shared: Arc<Shared>,
}

impl StaticSound {
	pub fn new(data: StaticSoundData, command_consumer: HeapConsumer<Command>) -> Self {
		let settings = data.settings;
		let starting_frame_index = starting_frame_index(settings, &data);
		let position = starting_frame_index as f64 / data.sample_rate as f64;
		let mut sound = Self {
			command_consumer,
			data,
			state: PlaybackState::Playing,
			when_to_start: if matches!(settings.start_time, StartTime::ClockTime(..)) {
				WhenToStart::Later
			} else {
				WhenToStart::Now
			},
			resampler: Resampler::new(starting_frame_index),
			current_frame_index: starting_frame_index,
			fractional_position: 0.0,
			volume: Tweener::new(settings.volume),
			playback_rate: Tweener::new(settings.playback_rate),
			panning: Tweener::new(settings.panning),
			volume_fade: create_volume_fade_tweener(settings),
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
		let max_frame_index = (self.data.frames.len() - 1)
			.try_into()
			.expect("sound is too long, cannot convert usize to i64");
		if let Some(LoopBehavior { start_position }) = self.data.settings.loop_behavior {
			let start_position = (start_position * self.data.sample_rate as f64) as i64;
			if self.current_frame_index >= max_frame_index {
				self.current_frame_index = start_position;
			} else {
				self.current_frame_index += 1;
			}
		} else {
			if self.current_frame_index >= max_frame_index {
				return true;
			} else {
				self.current_frame_index += 1;
			}
		}
		false
	}

	/// Decrements the playback position by 1 sample. Returns `true` if the end
	/// of the sound was reached (which in this case would be sample -1).
	fn decrement_position(&mut self) -> bool {
		let max_frame_index = (self.data.frames.len() - 1)
			.try_into()
			.expect("sound is too long, cannot convert usize to i64");
		if let Some(LoopBehavior { start_position }) = self.data.settings.loop_behavior {
			let start_position = (start_position * self.data.sample_rate as f64) as i64;
			if self.current_frame_index <= start_position {
				self.current_frame_index = max_frame_index;
			} else {
				self.current_frame_index -= 1;
			}
		} else {
			if self.current_frame_index == 0 {
				return true;
			} else {
				self.current_frame_index -= 1;
			}
		}
		false
	}

	/// Updates the current frame index by 1 and pushes a new sample to the resampler.
	fn update_position(&mut self) {
		if matches!(self.state, PlaybackState::Paused | PlaybackState::Stopped) {
			self.resampler
				.push_frame(Frame::ZERO, self.current_frame_index);
			return;
		}
		self.push_frame_to_resampler();
		let reached_end_of_sound = if self.playback_rate().is_sign_negative() {
			self.decrement_position()
		} else {
			self.increment_position()
		};
		if reached_end_of_sound {
			self.set_state(PlaybackState::Stopped);
		}
	}

	fn seek_to_index(&mut self, index: i64) {
		self.current_frame_index = index;
		let num_frames: i64 = self
			.data
			.frames
			.len()
			.try_into()
			.expect("sound is too long, cannot convert usize to i64");
		// if the seek index is past the end of the sound and the sound is
		// looping, wrap the seek point back into the sound
		if let Some(LoopBehavior { start_position }) = self.data.settings.loop_behavior {
			let start_position = (start_position * self.data.sample_rate as f64) as i64;
			while self.current_frame_index >= num_frames {
				self.current_frame_index -= num_frames - start_position;
			}
		// otherwise, stop the sound
		} else if self.current_frame_index >= num_frames {
			self.set_state(PlaybackState::Stopped);
		}
		// if the sound is playing, push a frame to the resample buffer
		// to make sure it doesn't get skipped
		if matches!(self.state, PlaybackState::Paused | PlaybackState::Stopped) {
			return;
		}
		self.push_frame_to_resampler();
	}

	fn push_frame_to_resampler(&mut self) {
		let num_frames: i64 = self
			.data
			.frames
			.len()
			.try_into()
			.expect("sound is too long, cannot convert usize to i64");
		let frame = if self.current_frame_index < 0 || self.current_frame_index >= num_frames {
			Frame::ZERO
		} else {
			let frame_index: usize = self
				.current_frame_index
				.try_into()
				.expect("cannot convert i64 into usize");
			(self.data.frames[frame_index]
				* self.volume_fade.value().as_amplitude() as f32
				* self.volume.value().as_amplitude() as f32)
				.panned(self.panning.value() as f32)
		};
		self.resampler.push_frame(frame, self.current_frame_index);
	}

	fn seek_by(&mut self, amount: f64) {
		let current_position = self.current_frame_index as f64 / self.data.sample_rate as f64;
		let position = current_position + amount;
		let index = (position * self.data.sample_rate as f64) as i64;
		self.seek_to_index(index);
	}

	fn seek_to(&mut self, position: f64) {
		let index = (position * self.data.sample_rate as f64) as i64;
		self.seek_to_index(index);
	}
}

impl Sound for StaticSound {
	fn output_destination(&mut self) -> OutputDestination {
		self.data.settings.output_destination
	}

	fn on_start_processing(&mut self) {
		let last_played_frame_position = self.resampler.current_frame_index();
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
					self.seek_by(amount);
				}
				Command::SeekTo(position) => {
					self.seek_to(position);
				}
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
				self.when_to_start =
					clock_info_provider.when_to_start(self.data.settings.start_time);
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

		// play back audio
		let out = self.resampler.get(self.fractional_position as f32);
		self.fractional_position += self.data.sample_rate as f64 * self.playback_rate().abs() * dt;
		while self.fractional_position >= 1.0 {
			self.fractional_position -= 1.0;
			self.update_position();
		}
		out
	}

	fn finished(&self) -> bool {
		self.state == PlaybackState::Stopped && self.resampler.outputting_silence()
	}
}

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

fn starting_frame_index(settings: StaticSoundSettings, data: &StaticSoundData) -> i64 {
	if settings.reverse {
		let position_seconds = data.duration().as_secs_f64() - settings.start_position;
		(position_seconds * data.sample_rate as f64) as i64 - 1
	} else {
		(settings.start_position * data.sample_rate as f64) as i64
	}
}

fn create_volume_fade_tweener(settings: StaticSoundSettings) -> Tweener<Volume> {
	if let Some(tween) = settings.fade_in_tween {
		let mut tweenable = Tweener::new(Volume::Decibels(Volume::MIN_DECIBELS));
		tweenable.set(Volume::Decibels(0.0), tween);
		tweenable
	} else {
		Tweener::new(Volume::Decibels(0.0))
	}
}
