mod resampler;

#[cfg(test)]
mod test;

use std::sync::{
	atomic::{AtomicU64, AtomicU8, Ordering},
	Arc,
};

use crate::{
	command::read_commands_into_parameters,
	frame::Frame,
	info::Info,
	playback_state_manager::PlaybackStateManager,
	sound::{transport::Transport, PlaybackRate, PlaybackState, Sound},
	tween::{Parameter, Tween},
	StartTime, Dbfs,
};

use self::resampler::Resampler;

use super::{data::StaticSoundData, frame_at_index, num_frames, CommandReaders};

pub(super) struct StaticSound {
	command_readers: CommandReaders,
	sample_rate: u32,
	frames: Arc<[Frame]>,
	slice: Option<(usize, usize)>,
	reverse: bool,
	playback_state_manager: PlaybackStateManager,
	start_time: StartTime,
	resampler: Resampler,
	transport: Transport,
	fractional_position: f64,
	volume: Parameter<Dbfs>,
	playback_rate: Parameter<PlaybackRate>,
	panning: Parameter,
	shared: Arc<Shared>,
}

impl StaticSound {
	#[must_use]
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
			sample_rate: data.sample_rate,
			frames: data.frames,
			slice: data.slice,
			reverse: data.settings.reverse,
			playback_state_manager: PlaybackStateManager::new(settings.fade_in_tween),
			start_time: settings.start_time,
			resampler: Resampler::new(starting_frame_index),
			transport,
			fractional_position: 0.0,
			volume: Parameter::new(settings.volume, Dbfs::MAX),
			playback_rate: Parameter::new(settings.playback_rate, PlaybackRate::Factor(1.0)),
			panning: Parameter::new(settings.panning, 0.5),
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

	fn update_shared_playback_state(&mut self) {
		self.shared
			.set_state(self.playback_state_manager.playback_state());
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

	#[must_use]
	fn is_playing_backwards(&self) -> bool {
		let mut is_playing_backwards = self.playback_rate.value().as_factor().is_sign_negative();
		if self.reverse {
			is_playing_backwards = !is_playing_backwards
		}
		is_playing_backwards
	}

	/// Updates the current frame index by 1 and pushes a new sample to the resampler.
	fn update_position(&mut self) {
		if !self.playback_state_manager.playback_state().is_advancing() {
			self.resampler
				.push_frame(Frame::ZERO, self.transport.position);
			return;
		}
		self.push_frame_to_resampler();
		if self.is_playing_backwards() {
			self.transport.decrement_position();
		} else {
			self.transport
				.increment_position(num_frames(&self.frames, self.slice));
		}
		if !self.transport.playing && self.resampler.outputting_silence() {
			self.playback_state_manager.mark_as_stopped();
			self.update_shared_playback_state();
		}
	}

	fn seek_to_index(&mut self, index: usize) {
		self.transport
			.seek_to(index, num_frames(&self.frames, self.slice));
		// if the sound is playing, push a frame to the resample buffer
		// to make sure it doesn't get skipped
		if self.playback_state_manager.playback_state().is_advancing() {
			self.push_frame_to_resampler();
		}
	}

	fn push_frame_to_resampler(&mut self) {
		let frame = if self.transport.playing {
			let frame_index: usize = self.transport.position;
			let fade_volume = self.playback_state_manager.fade_volume().as_amplitude();
			let volume = self.volume.value().as_amplitude();
			(frame_at_index(frame_index, &self.frames, self.slice).unwrap_or_default()
				* fade_volume * volume)
				.panned(self.panning.value() as f32)
		} else {
			Frame::ZERO
		};
		self.resampler.push_frame(frame, self.transport.position);
	}

	fn seek_by(&mut self, amount: f64) {
		let current_position = self.transport.position as f64 / self.sample_rate as f64;
		let position = current_position + amount;
		let index = (position * self.sample_rate as f64) as usize;
		self.seek_to_index(index);
	}

	fn seek_to(&mut self, position: f64) {
		let index = (position * self.sample_rate as f64) as usize;
		self.seek_to_index(index);
	}

	fn read_commands(&mut self) {
		read_commands_into_parameters!(self, volume, playback_rate, panning);
		if let Some(loop_region) = self.command_readers.set_loop_region.read() {
			self.transport.set_loop_region(
				loop_region,
				self.sample_rate,
				num_frames(&self.frames, self.slice),
			);
		}
		if let Some(tween) = self.command_readers.pause.read() {
			self.pause(tween);
		}
		if let Some((start_time, tween)) = self.command_readers.resume.read() {
			self.resume(start_time, tween);
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
	fn on_start_processing(&mut self) {
		let last_played_frame_position = self.resampler.current_frame_index();
		self.shared.position.store(
			(last_played_frame_position as f64 / self.sample_rate as f64).to_bits(),
			Ordering::SeqCst,
		);
		self.read_commands();
	}

	fn process(&mut self, dt: f64, info: &Info) -> Frame {
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

		// play back audio
		let out = self.resampler.get(self.fractional_position as f32);
		self.fractional_position +=
			self.sample_rate as f64 * self.playback_rate.value().as_factor().abs() * dt;
		while self.fractional_position >= 1.0 {
			self.fractional_position -= 1.0;
			self.update_position();
		}
		out
	}

	fn finished(&self) -> bool {
		self.playback_state_manager.playback_state() == PlaybackState::Stopped
	}
}

#[derive(Debug)]
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

	pub fn position(&self) -> f64 {
		f64::from_bits(self.position.load(Ordering::SeqCst))
	}
}
