use crate::{
	mixer::{SubTrackId, TrackIndex},
	parameter::{EaseDirection, Easing, Tween},
	Value,
};

use super::InstanceId;

/// A track index for an instance to play on.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize)
)]
pub enum InstanceTrackIndex {
	/// The default track for the sound.
	DefaultForSound,
	/// A manually set track index.
	Custom(TrackIndex),
}

impl Default for InstanceTrackIndex {
	fn default() -> Self {
		Self::DefaultForSound
	}
}

impl From<TrackIndex> for InstanceTrackIndex {
	fn from(index: TrackIndex) -> Self {
		Self::Custom(index)
	}
}

impl From<SubTrackId> for InstanceTrackIndex {
	fn from(id: SubTrackId) -> Self {
		Self::Custom(TrackIndex::Sub(id))
	}
}

/// A loop start point for an instance.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize)
)]
pub enum InstanceLoopStart {
	/// The default loop start point for the sound or arrangement.
	Default,
	/// No loop start point - the instance will not loop.
	None,
	/// A custom loop start point in seconds.
	Custom(f64),
}

impl Default for InstanceLoopStart {
	fn default() -> Self {
		Self::Default
	}
}

impl From<f64> for InstanceLoopStart {
	fn from(position: f64) -> Self {
		Self::Custom(position)
	}
}

impl From<Option<f64>> for InstanceLoopStart {
	fn from(option: Option<f64>) -> Self {
		match option {
			Some(position) => Self::Custom(position),
			None => Self::None,
		}
	}
}

/// Settings for an instance.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct InstanceSettings {
	pub id: InstanceId,
	/// The volume of the instance.
	pub volume: Value<f64>,
	/// The pitch of the instance, as a factor of the original pitch.
	pub pitch: Value<f64>,
	/// The panning of the instance (0 = hard left, 1 = hard right).
	pub panning: Value<f64>,
	/// The position to start playing the instance at (in seconds).
	pub start_position: f64,
	/// Whether to play the instance in reverse.
	pub reverse: bool,
	/// Whether to fade in the instance from silence, and if so,
	/// the tween to use.
	pub fade_in_tween: Option<Tween>,
	/// Whether the instance should loop, and if so, the position
	/// it should jump back to when it reaches the end.
	pub loop_start: InstanceLoopStart,
	/// Which track to play the instance on.
	pub track: InstanceTrackIndex,
}

impl InstanceSettings {
	/// Creates a new `InstanceSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	pub fn id(self, id: impl Into<InstanceId>) -> Self {
		Self {
			id: id.into(),
			..self
		}
	}

	/// Sets the volume of the instance.
	pub fn volume<V: Into<Value<f64>>>(self, volume: V) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/// Sets the pitch of the instance.
	pub fn pitch<P: Into<Value<f64>>>(self, pitch: P) -> Self {
		Self {
			pitch: pitch.into(),
			..self
		}
	}

	/// Sets the panning of the instance.
	pub fn panning<P: Into<Value<f64>>>(self, panning: P) -> Self {
		Self {
			panning: panning.into(),
			..self
		}
	}

	/// Sets where in the sound playback will start (in seconds).
	pub fn start_position(self, start_position: f64) -> Self {
		Self {
			start_position,
			..self
		}
	}

	/// Play the instance in reverse.
	pub fn reverse(self) -> Self {
		Self {
			reverse: true,
			..self
		}
	}

	/// Sets the tween the instance will use to fade in from silence.
	pub fn fade_in_tween(self, fade_in_tween: Tween) -> Self {
		Self {
			fade_in_tween: Some(fade_in_tween),
			..self
		}
	}

	/// Sets the portion of the sound that should be looped.
	pub fn loop_start<S: Into<InstanceLoopStart>>(self, start: S) -> Self {
		Self {
			loop_start: start.into(),
			..self
		}
	}

	/// Sets the track the instance will play on.
	pub fn track<T: Into<InstanceTrackIndex>>(self, track: T) -> Self {
		Self {
			track: track.into(),
			..self
		}
	}

	pub(crate) fn into_internal(
		self,
		duration: f64,
		default_loop_start: Option<f64>,
		default_track: TrackIndex,
	) -> InternalInstanceSettings {
		InternalInstanceSettings {
			id: self.id,
			volume: self.volume,
			pitch: self.pitch,
			panning: self.panning,
			start_position: if self.reverse {
				duration - self.start_position
			} else {
				self.start_position
			},
			reverse: self.reverse,
			fade_in_tween: self.fade_in_tween,
			loop_start: match self.loop_start {
				InstanceLoopStart::Default => default_loop_start,
				InstanceLoopStart::None => None,
				InstanceLoopStart::Custom(position) => Some(position),
			},
			track: match self.track {
				InstanceTrackIndex::DefaultForSound => default_track,
				InstanceTrackIndex::Custom(track) => track,
			},
		}
	}
}

impl Default for InstanceSettings {
	fn default() -> Self {
		Self {
			id: InstanceId::new(),
			volume: Value::Fixed(1.0),
			pitch: Value::Fixed(1.0),
			panning: Value::Fixed(0.5),
			start_position: 0.0,
			reverse: false,
			fade_in_tween: None,
			loop_start: InstanceLoopStart::default(),
			track: InstanceTrackIndex::default(),
		}
	}
}

pub(crate) struct InternalInstanceSettings {
	pub id: InstanceId,
	pub volume: Value<f64>,
	pub pitch: Value<f64>,
	pub panning: Value<f64>,
	pub start_position: f64,
	pub reverse: bool,
	pub fade_in_tween: Option<Tween>,
	pub loop_start: Option<f64>,
	pub track: TrackIndex,
}

/// Settings for pausing an instance.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct PauseInstanceSettings {
	/// Whether to fade the instance to silence, and if so,
	/// the tween to use.
	pub fade_tween: Option<Tween>,
}

impl PauseInstanceSettings {
	/// Creates a new `PauseInstanceSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the fade-out tween to use.
	pub fn fade_tween<T: Into<Option<Tween>>>(self, tween: T) -> Self {
		Self {
			fade_tween: tween.into(),
			..self
		}
	}
}

impl Default for PauseInstanceSettings {
	fn default() -> Self {
		Self {
			fade_tween: Some(Tween {
				duration: 0.001,
				easing: Easing::Linear,
				ease_direction: EaseDirection::In,
			}),
		}
	}
}

/// Settings for resuming an instance.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct ResumeInstanceSettings {
	/// Whether to fade in the instance from silence, and if so,
	/// the tween to use.
	pub fade_tween: Option<Tween>,
	/// Whether to seek the instance backwards to the playback
	/// position it was at when it was paused.
	pub rewind_to_pause_position: bool,
}

impl ResumeInstanceSettings {
	/// Creates a new `ResumeInstanceSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the fade-in tween to use.
	pub fn fade_tween<T: Into<Option<Tween>>>(self, tween: T) -> Self {
		Self {
			fade_tween: tween.into(),
			..self
		}
	}

	/// Sets whether to seek the instance backwards to the playback
	/// position it was at when it was paused.
	pub fn rewind_to_pause_position(self) -> Self {
		Self {
			rewind_to_pause_position: true,
			..self
		}
	}
}

impl Default for ResumeInstanceSettings {
	fn default() -> Self {
		Self {
			fade_tween: Some(Tween {
				duration: 0.001,
				easing: Easing::Linear,
				ease_direction: EaseDirection::In,
			}),
			rewind_to_pause_position: false,
		}
	}
}

/// Settings for stopping an instance.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct StopInstanceSettings {
	/// Whether to fade the instance to silence, and if so,
	/// the tween to use.
	pub fade_tween: Option<Tween>,
}

impl StopInstanceSettings {
	/// Creates a new `StopInstanceSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the fade-out tween to use.
	pub fn fade_tween<T: Into<Option<Tween>>>(self, tween: T) -> Self {
		Self {
			fade_tween: tween.into(),
			..self
		}
	}
}

impl Default for StopInstanceSettings {
	fn default() -> Self {
		Self {
			fade_tween: Some(Tween {
				duration: 0.001,
				easing: Easing::Linear,
				ease_direction: EaseDirection::In,
			}),
		}
	}
}
