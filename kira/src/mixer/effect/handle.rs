//! An interface for controlling effects.

use crate::{
	command::{
		producer::{CommandError, CommandProducer},
		MixerCommand,
	},
	mixer::TrackIndex,
};

use super::{EffectId, EffectSettings};

#[derive(Debug, Clone)]
/// Allows you to control an effect.
pub struct EffectHandle {
	id: EffectId,
	track_index: TrackIndex,
	enabled: bool,
	command_producer: CommandProducer,
}

impl EffectHandle {
	pub(crate) fn new(
		track_index: TrackIndex,
		settings: &EffectSettings,
		command_producer: CommandProducer,
	) -> Self {
		Self {
			id: settings.id,
			enabled: settings.enabled,
			track_index,
			command_producer,
		}
	}

	/// Returns the ID of the effect.
	pub fn id(&self) -> EffectId {
		self.id
	}

	/// Returns the track that contains this effect.
	pub fn track_index(&self) -> TrackIndex {
		self.track_index
	}

	/// Returns whether the effect is currently enabled.
	pub fn enabled(&self) -> bool {
		self.enabled
	}

	/// Sets whether the effect is currently enabled.
	pub fn set_enabled(&mut self, enabled: bool) -> Result<(), CommandError> {
		self.enabled = enabled;
		self.command_producer
			.push(MixerCommand::SetEffectEnabled(self.track_index, self.id, enabled).into())
	}
}
