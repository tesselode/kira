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
	modulator::value_provider::ModulatorValueProvider,
	sound::{
		transport::Transport, util::create_volume_fade_parameter, PlaybackRate, PlaybackState,
		Sound,
	},
	tween::{Parameter, Tween, Value},
	OutputDestination, StartTime, Volume,
};

use self::resampler::Resampler;

use super::{data::StaticSoundData, Command};

pub(super) struct StaticSound {
	command_consumer: HeapConsumer<Command>,
	data: StaticSoundData,
	state: PlaybackState,
	when_to_start: WhenToStart,
	resampler: Resampler,
	transport: Transport,
	fractional_position: f64,
	volume: Parameter<Volume>,
	playback_rate: Parameter<PlaybackRate>,
	panning: Parameter,
	volume_fade: Parameter<Volume>,
	shared: Arc<Shared>,
}

impl StaticSound {
	pub fn new(data: StaticSoundData, command_consumer: HeapConsumer<Command>) -> Self {
		let settings = data.settings;
		let transport = Transport::new(
			data.settings.playback_region,
			data.settings.loop_region,
			data.settings.reverse,
			data.sample_rate,
			data.frames.len(),
		);
		let starting_frame_index = transport.position;
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
			transport,
			fractional_position: 0.0,
			volume: Parameter::new(settings.volume, Volume::Amplitude(1.0)),
			playback_rate: Parameter::new(settings.playback_rate, PlaybackRate::Factor(1.0)),
			panning: Parameter::new(settings.panning, 0.5),
			volume_fade: create_volume_fade_parameter(settings.fade_in_tween),
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
		self.volume_fade.set(
			Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)),
			fade_out_tween,
		);
	}

	fn resume(&mut self, fade_in_tween: Tween) {
		self.set_state(PlaybackState::Playing);
		self.volume_fade
			.set(Value::Fixed(Volume::Decibels(0.0)), fade_in_tween);
	}

	fn stop(&mut self, fade_out_tween: Tween) {
		self.set_state(PlaybackState::Stopping);
		self.volume_fade.set(
			Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)),
			fade_out_tween,
		);
	}

	fn is_playing_backwards(&self) -> bool {
		let mut is_playing_backwards = self.playback_rate.value().as_factor().is_sign_negative();
		if self.data.settings.reverse {
			is_playing_backwards = !is_playing_backwards
		}
		is_playing_backwards
	}

	/// Updates the current frame index by 1 and pushes a new sample to the resampler.
	fn update_position(&mut self) {
		if matches!(self.state, PlaybackState::Paused | PlaybackState::Stopped) {
			self.resampler
				.push_frame(Frame::ZERO, self.transport.position);
			return;
		}
		self.push_frame_to_resampler();
		if self.is_playing_backwards() {
			self.transport.decrement_position();
		} else {
			self.transport.increment_position();
		}
		if !self.transport.playing {
			self.set_state(PlaybackState::Stopped);
		}
	}

	fn seek_to_index(&mut self, index: i64) {
		self.transport.seek_to(index);
		// if the sound is playing, push a frame to the resample buffer
		// to make sure it doesn't get skipped
		if !matches!(self.state, PlaybackState::Paused | PlaybackState::Stopped) {
			self.push_frame_to_resampler();
		}
	}

	fn push_frame_to_resampler(&mut self) {
		let num_frames: i64 = self
			.data
			.frames
			.len()
			.try_into()
			.expect("sound is too long, cannot convert usize to i64");
		let frame = if self.transport.position < 0 || self.transport.position >= num_frames {
			Frame::ZERO
		} else {
			let frame_index: usize = self
				.transport
				.position
				.try_into()
				.expect("cannot convert i64 into usize");
			(self.data.frames[frame_index]
				* self.volume_fade.value().as_amplitude() as f32
				* self.volume.value().as_amplitude() as f32)
				.panned(self.panning.value() as f32)
		};
		self.resampler.push_frame(frame, self.transport.position);
	}

	fn seek_by(&mut self, amount: f64) {
		let current_position = self.transport.position as f64 / self.data.sample_rate as f64;
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
		// Update our reported position if we aren't paused.
		// If the sound is paused but the user seeks around, we'll repeatedly report
		// where we *were*, but not where we now are.
		if !matches!(self.state, PlaybackState::Paused) {
			let last_played_frame_position = self.resampler.current_frame_index();

			self.shared.position.store(
				(last_played_frame_position as f64 / self.data.sample_rate as f64).to_bits(),
				Ordering::SeqCst,
			);
		}

		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetVolume(volume, tween) => self.volume.set(volume, tween),
				Command::SetPlaybackRate(playback_rate, tween) => {
					self.playback_rate.set(playback_rate, tween)
				}
				Command::SetPanning(panning, tween) => self.panning.set(panning, tween),
				Command::SetPlaybackRegion(playback_region) => self.transport.set_playback_region(
					playback_region,
					self.data.sample_rate,
					self.data.frames.len(),
				),
				Command::SetLoopRegion(loop_region) => self.transport.set_loop_region(
					loop_region,
					self.data.sample_rate,
					self.data.frames.len(),
				),
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
		self.fractional_position +=
			self.data.sample_rate as f64 * self.playback_rate.value().as_factor().abs() * dt;
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
