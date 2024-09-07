use crate::{
	info::Info,
	sound::PlaybackState,
	tween::{Parameter, Tween, Value},
	StartTime, Volume,
};

pub(crate) struct PlaybackStateManager {
	state: State,
	volume_fade: Parameter<Volume>,
}

impl PlaybackStateManager {
	pub fn new(fade_in_tween: Option<Tween>) -> Self {
		Self {
			state: State::Playing,
			volume_fade: fade_in_tween
				.map(|tween| {
					let mut parameter = Parameter::new(
						Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)),
						Volume::Decibels(Volume::MIN_DECIBELS),
					);
					parameter.set(Value::Fixed(Volume::Decibels(0.0)), tween);
					parameter
				})
				.unwrap_or_else(|| {
					Parameter::new(Value::Fixed(Volume::Decibels(0.0)), Volume::Decibels(0.0))
				}),
		}
	}

	pub fn fade_volume(&self) -> Volume {
		self.volume_fade.value()
	}

	pub fn playback_state(&self) -> PlaybackState {
		match self.state {
			State::Playing => PlaybackState::Playing,
			State::Pausing => PlaybackState::Pausing,
			State::Paused => PlaybackState::Paused,
			State::WaitingToResume { .. } => PlaybackState::WaitingToResume,
			State::Resuming => PlaybackState::Resuming,
			State::Stopping => PlaybackState::Stopping,
			State::Stopped => PlaybackState::Stopped,
		}
	}

	pub fn pause(&mut self, fade_out_tween: Tween) {
		if let State::Stopped = &self.state {
			return;
		}
		self.state = State::Pausing;
		self.volume_fade.set(
			Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)),
			fade_out_tween,
		);
	}

	pub fn resume(&mut self, start_time: StartTime, fade_in_tween: Tween) {
		if let State::Stopped = &self.state {
			return;
		}
		if let StartTime::Immediate = start_time {
			self.state = State::Resuming;
			self.volume_fade
				.set(Value::Fixed(Volume::Decibels(0.0)), fade_in_tween);
		} else {
			self.state = State::WaitingToResume {
				start_time,
				fade_in_tween,
			};
		}
	}

	pub fn stop(&mut self, fade_out_tween: Tween) {
		if let State::Stopped = &self.state {
			return;
		}
		self.state = State::Stopping;
		self.volume_fade.set(
			Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)),
			fade_out_tween,
		);
	}

	pub fn mark_as_stopped(&mut self) {
		self.state = State::Stopped;
	}

	pub fn update(&mut self, dt: f64, info: &Info) -> ChangedPlaybackState {
		match &mut self.state {
			State::Playing => {}
			State::Pausing => {
				let finished = self.volume_fade.update(dt, info);
				if finished {
					self.state = State::Paused;
					return true;
				}
			}
			State::Paused => {}
			State::WaitingToResume {
				start_time,
				fade_in_tween,
			} => {
				let will_never_start = start_time.update(dt, info);
				if will_never_start {
					self.state = State::Stopped;
					return true;
				}
				if *start_time == StartTime::Immediate {
					let fade_in_tween = *fade_in_tween;
					self.resume(StartTime::Immediate, fade_in_tween);
					return true;
				}
			}
			State::Resuming => {
				let finished = self.volume_fade.update(dt, info);
				if finished {
					self.state = State::Playing;
					return true;
				}
			}
			State::Stopping => {
				let finished = self.volume_fade.update(dt, info);
				if finished {
					self.state = State::Stopped;
					return true;
				}
			}
			State::Stopped => {}
		}
		false
	}
}

pub type ChangedPlaybackState = bool;

enum State {
	Playing,
	Pausing,
	Paused,
	WaitingToResume {
		start_time: StartTime,
		fade_in_tween: Tween,
	},
	Resuming,
	Stopping,
	Stopped,
}
