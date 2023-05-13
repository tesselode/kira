use std::{collections::HashSet, error::Error, fmt::Display, sync::Arc};

use crate::{
	error::CommandError,
	manager::command::{producer::CommandProducer, Command, MixerCommand},
	tween::{Tween, Value},
	Volume,
};

use super::{TrackId, TrackShared};

/// Errors that can occur when changing the volume of a track route.
#[derive(Debug)]
pub enum SetRouteError {
	/// Cannot change the volume of a track route that did not exist originally.
	NonexistentRoute,
	/// An error occurred when sending a command to the audio renderer.
	CommandError(CommandError),
}

impl Display for SetRouteError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			SetRouteError::NonexistentRoute => f.write_str(
				"Cannot change the volume of a track route that did not exist originally",
			),
			SetRouteError::CommandError(error) => error.fmt(f),
		}
	}
}

impl Error for SetRouteError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			SetRouteError::CommandError(error) => Some(error),
			_ => None,
		}
	}
}

/// Controls a mixer track.
///
/// When a [`TrackHandle`] is dropped, the corresponding mixer
/// track will be removed.
pub struct TrackHandle {
	pub(crate) id: TrackId,
	pub(crate) shared: Option<Arc<TrackShared>>,
	pub(crate) command_producer: CommandProducer,
	pub(crate) existing_routes: HashSet<TrackId>,
}

impl TrackHandle {
	/// Returns the unique identifier for the mixer track.
	pub fn id(&self) -> TrackId {
		self.id
	}

	/// Sets the (post-effects) volume of the mixer track.
	pub fn set_volume(
		&self,
		volume: impl Into<Value<Volume>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Mixer(MixerCommand::SetTrackVolume(
				self.id,
				volume.into(),
				tween,
			)))
	}

	/// Sets the volume of this track's route to another track.
	///
	/// This can only be used to change the volume of existing routes,
	/// not to add new routes.
	pub fn set_route(
		&self,
		to: impl Into<TrackId>,
		volume: impl Into<Value<Volume>>,
		tween: Tween,
	) -> Result<(), SetRouteError> {
		let to = to.into();
		if !self.existing_routes.contains(&to) {
			return Err(SetRouteError::NonexistentRoute);
		}
		self.command_producer
			.push(Command::Mixer(MixerCommand::SetTrackRoutes {
				from: self.id,
				to,
				volume: volume.into(),
				tween,
			}))
			.map_err(SetRouteError::CommandError)
	}
}

impl Drop for TrackHandle {
	fn drop(&mut self) {
		if let Some(shared) = &self.shared {
			shared.mark_for_removal();
		}
	}
}
