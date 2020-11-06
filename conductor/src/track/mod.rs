pub mod effect;
pub(crate) mod effect_slot;
mod id;

pub use id::SubTrackId;

use indexmap::IndexMap;

use crate::{manager::backend::parameters::Parameters, stereo_sample::StereoSample};

use self::{
	effect::{Effect, EffectId, EffectSettings},
	effect_slot::EffectSlot,
};

/// Represents a mixer track.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone)]
pub struct TrackSettings {
	pub volume: f64,
}

impl Default for TrackSettings {
	fn default() -> Self {
		Self { volume: 1.0 }
	}
}

pub(crate) struct Track {
	volume: f64,
	effect_slots: IndexMap<EffectId, EffectSlot>,
	input: StereoSample,
}

impl Track {
	pub fn new(settings: TrackSettings) -> Self {
		Self {
			volume: settings.volume,
			effect_slots: IndexMap::new(),
			input: StereoSample::from_mono(0.0),
		}
	}

	pub fn add_effect(&mut self, id: EffectId, effect: Box<dyn Effect>, settings: EffectSettings) {
		self.effect_slots
			.insert(id, EffectSlot::new(effect, settings));
	}

	pub fn remove_effect(&mut self, id: EffectId) -> Option<EffectSlot> {
		self.effect_slots.remove(&id)
	}

	pub fn add_input(&mut self, input: StereoSample) {
		self.input += input;
	}

	pub fn process(&mut self, dt: f64, parameters: &Parameters) -> StereoSample {
		let mut input = self.input;
		self.input = StereoSample::from_mono(0.0);
		for (_, effect_slot) in &mut self.effect_slots {
			input = effect_slot.process(dt, input, parameters);
		}
		input * (self.volume as f32)
	}
}
