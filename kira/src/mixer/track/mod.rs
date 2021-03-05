pub mod handle;
pub mod sends;
pub mod settings;

pub use settings::*;

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

impl From<&SubTrackHandle> for SubTrackId {
	fn from(handle: &SubTrackHandle) -> Self {
		handle.id()
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

impl From<&SendTrackHandle> for SendTrackId {
	fn from(handle: &SendTrackHandle) -> Self {
		handle.id()
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

pub(crate) enum TrackKind {
	Main,
	Sub {
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
	pub fn new_main_track() -> Self {
		Self {
			kind: TrackKind::Main,
			volume: 1.0,
			effect_slots: StaticIndexMap::new(0),
			input: Frame::from_mono(0.0),
		}
	}

	pub fn new_sub_track(id: SubTrackId, settings: SubTrackSettings) -> Self {
		Self {
			kind: TrackKind::Sub {
				id,
				parent_track: settings.parent_track,
				sends: settings.sends,
			},
			volume: settings.volume,
			effect_slots: StaticIndexMap::new(settings.num_effects),
			input: Frame::from_mono(0.0),
		}
	}

	pub fn new_send_track(id: SendTrackId, settings: SendTrackSettings) -> Self {
		Self {
			kind: TrackKind::Send { id },
			volume: settings.volume,
			effect_slots: StaticIndexMap::new(settings.num_effects),
			input: Frame::from_mono(0.0),
		}
	}

	pub fn parent_track(&self) -> Option<TrackIndex> {
		match &self.kind {
			TrackKind::Main => None,
			TrackKind::Sub { parent_track, .. } => Some(*parent_track),
			TrackKind::Send { .. } => Some(TrackIndex::Main),
		}
	}

	pub fn kind(&self) -> &TrackKind {
		&self.kind
	}

	pub fn add_effect(
		&mut self,
		id: EffectId,
		effect: Owned<Box<dyn Effect>>,
		settings: EffectSettings,
	) {
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
		if let TrackKind::Sub { sends, .. } = &mut self.kind {
			sends.update(parameters);
		}
		let mut input = self.input;
		self.input = Frame::from_mono(0.0);
		for (_, effect_slot) in &mut self.effect_slots {
			input = effect_slot.process(dt, input, parameters);
		}
		input * (self.volume as f32)
	}
}
