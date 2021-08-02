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
	pub(crate) shared: Arc<TrackShared>,
	pub(crate) command_producer: CommandProducer,
}

impl TrackHandle {
	pub fn id(&self) -> TrackId {
		self.id
	}

	pub fn set_volume(&mut self, volume: impl Into<Value>) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Mixer(MixerCommand::SetTrackVolume(
				self.id,
				volume.into(),
			)))
	}

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
		self.shared.mark_for_removal();
	}
}
