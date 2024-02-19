#[cfg(test)]
mod test;

use std::{
	sync::{
		atomic::{AtomicU8, Ordering},
		Arc,
	},
	time::Duration,
};

use crate::{
	clock::clock_info::{ClockInfoProvider, WhenToStart},
	dsp::{interpolate_frame, Frame},
	modulator::value_provider::ModulatorValueProvider,
	tween::{Parameter, Tween, Value},
	OutputDestination, StartTime, Volume,
};

use super::{CommonSoundSettings, PlaybackState, Sound};

pub(crate) struct SoundWrapper {
	sound: Box<dyn Sound>,
	state: PlaybackState,
	start_time: StartTime,
	volume: Parameter<Volume>,
	panning: Parameter,
	volume_fade: Parameter<Volume>,
	output_destination: OutputDestination,
	time_since_last_frame: f64,
	resample_buffer: [Frame; 4],
	shared: SoundWrapperShared,
}

impl SoundWrapper {
	pub fn new(
		sound: Box<dyn Sound>,
		settings: CommonSoundSettings,
		shared: SoundWrapperShared,
	) -> Self {
		Self {
			sound,
			state: PlaybackState::Playing,
			start_time: settings.start_time,
			volume: Parameter::new(settings.volume, Volume::Amplitude(1.0)),
			panning: Parameter::new(settings.panning, 0.5),
			volume_fade: {
				let fade_in_tween = settings.fade_in_tween;
				if let Some(tween) = fade_in_tween {
					let mut tweenable = Parameter::new(
						Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)),
						Volume::Decibels(Volume::MIN_DECIBELS),
					);
					tweenable.set(Value::Fixed(Volume::Decibels(0.0)), tween);
					tweenable
				} else {
					Parameter::new(Value::Fixed(Volume::Decibels(0.0)), Volume::Decibels(0.0))
				}
			},
			output_destination: settings.output_destination,
			time_since_last_frame: 0.0,
			resample_buffer: [Frame::from_mono(0.0); 4],
			shared,
		}
	}

	pub fn output_destination(&self) -> OutputDestination {
		self.output_destination
	}

	pub fn finished(&self) -> bool {
		self.sound.finished() || self.state == PlaybackState::Stopped
	}

	pub fn on_start_processing(&mut self) {
		self.sound.on_start_processing();
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		// update parameters
		self.volume
			.update(dt, clock_info_provider, modulator_value_provider);
		self.panning
			.update(dt, clock_info_provider, modulator_value_provider);
		if self
			.volume_fade
			.update(dt, clock_info_provider, modulator_value_provider)
		{
			match self.state {
				PlaybackState::Pausing => self.set_state(PlaybackState::Paused),
				PlaybackState::Stopping => {
					self.set_state(PlaybackState::Stopped);
					self.sound.on_stop();
				}
				_ => {}
			}
		}

		if matches!(self.state, PlaybackState::Paused | PlaybackState::Stopped) {
			return Frame::ZERO;
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
						self.stop(Tween::default());
						false
					}
				}
			}
		};
		if !started {
			return Frame::ZERO;
		}

		// collect audio output from the underlying sound
		self.time_since_last_frame += dt;
		while self.time_since_last_frame >= 1.0 / self.sound.sample_rate() {
			self.time_since_last_frame -= 1.0 / self.sound.sample_rate();
			for i in 0..self.resample_buffer.len() - 1 {
				self.resample_buffer[i] = self.resample_buffer[i + 1];
			}
			self.resample_buffer[self.resample_buffer.len() - 1] = self
				.sound
				.process(clock_info_provider, modulator_value_provider);
		}

		if self.sound.finished() {
			self.set_state(PlaybackState::Stopped);
		}

		// play back audio
		interpolate_frame(
			self.resample_buffer[0],
			self.resample_buffer[1],
			self.resample_buffer[2],
			self.resample_buffer[3],
			(self.time_since_last_frame * self.sound.sample_rate()) as f32,
		)
		.panned(self.panning.value() as f32)
			* self.volume.value().as_amplitude() as f32
			* self.volume_fade.value().as_amplitude() as f32
	}

	pub fn pause(&mut self, fade_out_tween: Tween) {
		self.set_state(PlaybackState::Pausing);
		self.volume_fade.set(
			Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)),
			fade_out_tween,
		);
	}

	pub fn resume(&mut self, fade_in_tween: Tween) {
		self.set_state(PlaybackState::Playing);
		self.volume_fade
			.set(Value::Fixed(Volume::Decibels(0.0)), fade_in_tween);
	}

	pub fn stop(&mut self, fade_out_tween: Tween) {
		self.set_state(PlaybackState::Stopping);
		self.volume_fade.set(
			Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)),
			fade_out_tween,
		);
	}

	pub fn set_volume(&mut self, target: Value<Volume>, tween: Tween) {
		self.volume_fade.set(target, tween)
	}

	pub fn set_panning(&mut self, target: Value<f64>, tween: Tween) {
		self.panning.set(target, tween)
	}

	fn set_state(&mut self, state: PlaybackState) {
		self.state = state;
		self.shared.state.store(state as u8, Ordering::SeqCst);
	}
}

#[derive(Debug, Clone)]
pub(crate) struct SoundWrapperShared {
	pub(crate) state: Arc<AtomicU8>,
}

impl SoundWrapperShared {
	pub(crate) fn new() -> Self {
		Self {
			state: Arc::new(AtomicU8::new(PlaybackState::Playing as u8)),
		}
	}
}