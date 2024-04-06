mod resampler;

#[cfg(test)]
mod test;

use std::{
	sync::{
		atomic::{AtomicU64, AtomicU8, Ordering},
		Arc,
	},
	time::Duration,
};

use crate::{
	clock::clock_info::{ClockInfoProvider, WhenToStart},
	dsp::Frame,
	modulator::value_provider::ModulatorValueProvider,
	read_commands_into_parameters,
	sound::{
		transport::Transport, util::create_volume_fade_parameter, PlaybackRate, PlaybackState,
		Sound,
	},
	tween::{Parameter, Tween, Value},
	OutputDestination, StartTime, Volume,
};

use self::resampler::Resampler;

use super::{data::StaticSoundData, CommandReaders};

pub(super) struct StaticSound {
	command_readers: CommandReaders,
	data: StaticSoundData,
	state: PlaybackState,
	start_time: StartTime,
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
	pub fn new(data: StaticSoundData, command_readers: CommandReaders) -> Self {
		let settings = data.settings;
		let transport = Transport::new(
			data.settings.start_position.into_samples(data.sample_rate),
			data.settings.loop_region,
			data.settings.reverse,
			data.sample_rate,
			data.num_frames(),
		);
		let starting_frame_index = transport.position;
		let position = starting_frame_index as f64 / data.sample_rate as f64;
		let mut sound = Self {
			command_readers,
			data,
			state: PlaybackState::Playing,
			start_time: settings.start_time,
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
			self.transport.increment_position(self.data.num_frames());
		}
		if !self.transport.playing {
			self.set_state(PlaybackState::Stopped);
		}
	}

	fn seek_to_index(&mut self, index: usize) {
		self.transport.seek_to(index, self.data.num_frames());
		// if the sound is playing, push a frame to the resample buffer
		// to make sure it doesn't get skipped
		if !matches!(self.state, PlaybackState::Paused | PlaybackState::Stopped) {
			self.push_frame_to_resampler();
		}
	}

	fn push_frame_to_resampler(&mut self) {
		let num_frames = self.data.num_frames();
		let frame = if self.transport.position >= num_frames {
			Frame::ZERO
		} else {
			let frame_index: usize = self.transport.position;
			(self.data.frame_at_index(frame_index).unwrap_or_default()
				* self.volume_fade.value().as_amplitude() as f32
				* self.volume.value().as_amplitude() as f32)
				.panned(self.panning.value() as f32)
		};
		self.resampler.push_frame(frame, self.transport.position);
	}

	fn seek_by(&mut self, amount: f64) {
		let current_position = self.transport.position as f64 / self.data.sample_rate as f64;
		let position = current_position + amount;
		let index = (position * self.data.sample_rate as f64) as usize;
		self.seek_to_index(index);
	}

	fn seek_to(&mut self, position: f64) {
		let index = (position * self.data.sample_rate as f64) as usize;
		self.seek_to_index(index);
	}

	fn read_commands(&mut self) {
		read_commands_into_parameters!(self, volume, playback_rate, panning);
		if let Some(loop_region) = self.command_readers.set_loop_region.read() {
			self.transport.set_loop_region(
				loop_region,
				self.data.sample_rate,
				self.data.num_frames(),
			);
		}
		if let Some(tween) = self.command_readers.pause.read() {
			self.pause(tween);
		}
		if let Some(tween) = self.command_readers.resume.read() {
			self.resume(tween);
		}
		if let Some(tween) = self.command_readers.stop.read() {
			self.stop(tween);
		}
		if let Some(amount) = self.command_readers.seek_by.read() {
			self.seek_by(amount);
		}
		if let Some(position) = self.command_readers.seek_to.read() {
			self.seek_to(position);
		}
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
