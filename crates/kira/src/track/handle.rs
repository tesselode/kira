use std::sync::Arc;

use crate::{
	error::CommandError,
	manager::command::{producer::CommandProducer, Command, MixerCommand},
	value::Value,
};

use super::{TrackId, TrackShared};

/// Controls a mixer track.
///
/// When a [`TrackHandle`] is dropped, the corresponding mixer
/// track will be removed.
pub struct TrackHandle {
	pub(crate) id: TrackId,
	pub(crate) shared: Option<Arc<TrackShared>>,
	pub(crate) command_producer: CommandProducer,
}

impl TrackHandle {
	/// Returns the unique identifier for the mixer track.
	pub fn id(&self) -> TrackId {
		self.id
	}

	/// Sets the (post-effects) volume of the mixer track.
	pub fn set_volume(&mut self, volume: impl Into<Value>) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Mixer(MixerCommand::SetTrackVolume(
				self.id,
				volume.into(),
			)))
	}

	/// Sets the (post-effects) panning of the mixer track, where
	/// 0.0 is hard left and 1.0 is hard right.
	pub fn set_panning(&mut self, panning: impl Into<Value>) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Mixer(MixerCommand::SetTrackPanning(
				self.id,
				panning.into(),
			)))
	}
}

impl Drop for TrackHandle {
	fn drop(&mut self) {
		if let Some(shared) = &self.shared {
			shared.mark_for_removal();
		}
	}
}
