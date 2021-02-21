pub mod handle;
pub mod sends;

use basedrop::Owned;
use handle::{SendTrackHandle, SubTrackHandle};
use sends::TrackSends;
use uuid::Uuid;

use crate::{frame::Frame, parameter::Parameters, static_container::index_map::StaticIndexMap};

use super::{
	effect::{Effect, EffectId, EffectSettings},
	effect_slot::EffectSlot,
};

/// A unique identifier for a sub-track.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
pub struct SubTrackId {
	uuid: Uuid,
}

impl SubTrackId {
	pub(crate) fn new() -> Self {
		Self {
			uuid: Uuid::new_v4(),
		}
	}
}

/// A unique identifier for a send track.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
pub struct SendTrackId {
	uuid: Uuid,
}

impl SendTrackId {
	pub(crate) fn new() -> Self {
		Self {
			uuid: Uuid::new_v4(),
		}
	}
}

/// An identifier for a mixer track.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize)
)]
pub enum TrackIndex {
	/// The main track.
	///
	/// All sub-tracks are sent to the main track as input,
	/// and the output of the main track is what you hear.
	Main,
	/// A sub-track.
	///
	/// Sub-tracks are useful for adjusting the volumes of
	/// and applying effects to certain kinds of sounds.
	/// For example, in a game, you may have one sub-track
	/// for sound effects and another for music.
	Sub(SubTrackId),
	/// A send track.
	///
	/// Send tracks are useful for routing multiple sub-tracks
	/// into the same set of effects. This can save processing
	/// power and avoid redundant effect configuration.
	Send(SendTrackId),
}

impl Default for TrackIndex {
	fn default() -> Self {
		Self::Main
	}
}

impl From<SubTrackId> for TrackIndex {
	fn from(id: SubTrackId) -> Self {
		Self::Sub(id)
	}
}

impl From<&SubTrackHandle> for TrackIndex {
	fn from(handle: &SubTrackHandle) -> Self {
		TrackIndex::Sub(handle.id())
	}
}

impl From<SendTrackId> for TrackIndex {
	fn from(id: SendTrackId) -> Self {
		Self::Send(id)
	}
}

impl From<&SendTrackHandle> for TrackIndex {
	fn from(handle: &SendTrackHandle) -> Self {
		TrackIndex::Send(handle.id())
	}
}

/// Settings for a mixer sub-track.
#[derive(Debug, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct SubTrackSettings {
	/// The unique identifier for the track.
	pub id: SubTrackId,
	/// The track that this track's output will be routed to.
	pub parent_track: TrackIndex,
	/// The send tracks that this track will be routed to (in
	/// addition to the parent track).
	pub sends: TrackSends,
	/// The volume of the track.
	pub volume: f64,
	/// The maximum number of effects this track can hold.
	pub num_effects: usize,
}

impl SubTrackSettings {
	/// Creates a new `TrackSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the unique identifier for the track.
	pub fn id(self, id: impl Into<SubTrackId>) -> Self {
		Self {
			id: id.into(),
			..self
		}
	}

	/// Sets the track that this track's output will be routed to.
	pub fn parent_track(self, parent_track: impl Into<TrackIndex>) -> Self {
		Self {
			parent_track: parent_track.into(),
			..self
		}
	}

	/// Sets the send tracks that this track will be routed to (in
	/// addition to the parent track).
	pub fn sends(self, sends: TrackSends) -> Self {
		Self { sends, ..self }
	}

	/// Sets the volume of the track.
	pub fn volume(self, volume: impl Into<f64>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/// Sets the maximum number of effects this track can hold.
	pub fn num_effects(self, num_effects: usize) -> Self {
		Self {
			num_effects,
			..self
		}
	}
}

impl Default for SubTrackSettings {
	fn default() -> Self {
		Self {
			id: SubTrackId::new(),
			parent_track: TrackIndex::Main,
			sends: TrackSends::new(),
			volume: 1.0,
			num_effects: 10,
		}
	}
}

/// Settings for a mixer send-track.
#[derive(Debug, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct SendTrackSettings {
	/// The unique identifier for the track.
	pub id: SendTrackId,
	/// The volume of the track.
	pub volume: f64,
	/// The maximum number of effects this track can hold.
	pub num_effects: usize,
}

impl SendTrackSettings {
	/// Creates a new `TrackSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the unique identifier for the track.
	pub fn id(self, id: impl Into<SendTrackId>) -> Self {
		Self {
			id: id.into(),
			..self
		}
	}

	/// Sets the volume of the track.
	pub fn volume(self, volume: impl Into<f64>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/// Sets the maximum number of effects this track can hold.
	pub fn num_effects(self, num_effects: usize) -> Self {
		Self {
			num_effects,
			..self
		}
	}
}

impl Default for SendTrackSettings {
	fn default() -> Self {
		Self {
			id: SendTrackId::new(),
			volume: 1.0,
			num_effects: 10,
		}
	}
}

pub(crate) enum TrackKind {
	Normal {
		id: SubTrackId,
		parent_track: TrackIndex,
		sends: TrackSends,
	},
	Send {
		id: SendTrackId,
	},
}

pub(crate) struct Track {
	kind: TrackKind,
	volume: f64,
	effect_slots: StaticIndexMap<EffectId, EffectSlot>,
	input: Frame,
}

impl Track {
	pub fn new_normal_track(settings: SubTrackSettings) -> Self {
		Self {
			kind: TrackKind::Normal {
				id: settings.id,
				parent_track: settings.parent_track,
				sends: settings.sends,
			},
			volume: settings.volume,
			effect_slots: StaticIndexMap::new(settings.num_effects),
			input: Frame::from_mono(0.0),
		}
	}

	pub fn new_send_track(settings: SendTrackSettings) -> Self {
		Self {
			kind: TrackKind::Send { id: settings.id },
			volume: settings.volume,
			effect_slots: StaticIndexMap::new(settings.num_effects),
			input: Frame::from_mono(0.0),
		}
	}

	pub fn parent_track(&self) -> TrackIndex {
		match &self.kind {
			TrackKind::Normal { parent_track, .. } => *parent_track,
			TrackKind::Send { id } => TrackIndex::Main,
		}
	}

	pub fn kind(&self) -> &TrackKind {
		&self.kind
	}

	pub fn add_effect(&mut self, effect: Owned<Box<dyn Effect>>, settings: EffectSettings) {
		let id = settings.id;
		let effect_slot = EffectSlot::new(effect, settings);
		self.effect_slots.try_insert(id, effect_slot).ok();
	}

	pub fn effect_mut(&mut self, id: EffectId) -> Option<&mut EffectSlot> {
		self.effect_slots.get_mut(&id)
	}

	pub fn remove_effect(&mut self, id: EffectId) {
		self.effect_slots.remove(&id);
	}

	pub fn add_input(&mut self, input: Frame) {
		self.input += input;
	}

	pub fn process(&mut self, dt: f64, parameters: &Parameters) -> Frame {
		let mut input = self.input;
		self.input = Frame::from_mono(0.0);
		for (_, effect_slot) in &mut self.effect_slots {
			input = effect_slot.process(dt, input, parameters);
		}
		input * (self.volume as f32)
	}
}
