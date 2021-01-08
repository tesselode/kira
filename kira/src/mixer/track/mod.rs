mod handle;

pub use handle::TrackHandle;
use uuid::Uuid;

use indexmap::IndexMap;

use crate::{frame::Frame, parameter::Parameters, util::generate_uuid};

use super::{
	effect::{Effect, EffectId, EffectSettings},
	effect_slot::EffectSlot,
};

/**
A unique identifier for a sub-track.

You cannot create this manually - a `SubTrackId` is created
when you create a sub-track with an [`AudioManager`](crate::manager::AudioManager).
*/
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
			uuid: generate_uuid(),
		}
	}
}

/// Represents a mixer track.
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

impl From<&TrackHandle> for TrackIndex {
	fn from(handle: &TrackHandle) -> Self {
		handle.index()
	}
}

/// Settings for a mixer track.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct TrackSettings {
	/// The unique identifier for the track.
	pub id: SubTrackId,
	/// The volume of the track.
	pub volume: f64,
}

impl TrackSettings {
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

	/// Sets the volume of the track.
	pub fn volume(self, volume: impl Into<f64>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}
}

impl Default for TrackSettings {
	fn default() -> Self {
		Self {
			id: SubTrackId::new(),
			volume: 1.0,
		}
	}
}

#[derive(Debug)]
pub(crate) struct Track {
	id: SubTrackId,
	volume: f64,
	effect_slots: IndexMap<EffectId, EffectSlot>,
	input: Frame,
}

impl Track {
	pub fn new(settings: TrackSettings) -> Self {
		Self {
			id: settings.id,
			volume: settings.volume,
			effect_slots: IndexMap::new(),
			input: Frame::from_mono(0.0),
		}
	}

	pub fn id(&self) -> SubTrackId {
		self.id
	}

	pub fn add_effect(&mut self, effect: Box<dyn Effect>, settings: EffectSettings) {
		let effect_slot = EffectSlot::new(effect, settings);
		self.effect_slots.insert(effect_slot.id(), effect_slot);
	}

	pub fn remove_effect(&mut self, id: EffectId) -> Option<EffectSlot> {
		self.effect_slots.remove(&id)
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
