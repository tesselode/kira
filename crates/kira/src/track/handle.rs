use std::{collections::HashMap, error::Error, fmt::Display, sync::Arc};

use crate::{
	command::{CommandWriter, ValueChangeCommand},
	tween::{Tween, Value},
	Volume,
};

use super::{TrackId, TrackShared};

/// Cannot change the volume of a track route that did not exist originally.
#[derive(Debug)]
pub struct NonexistentRoute;

impl Display for NonexistentRoute {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("Cannot change the volume of a track route that did not exist originally")
	}
}

impl Error for NonexistentRoute {}

/// Controls a mixer track.
///
/// When a [`TrackHandle`] is dropped, the corresponding mixer
/// track will be removed.
pub struct TrackHandle {
	pub(crate) id: TrackId,
	pub(crate) shared: Option<Arc<TrackShared>>,
	pub(crate) set_volume_command_writer: CommandWriter<ValueChangeCommand<Volume>>,
	pub(crate) route_set_volume_command_writers:
		HashMap<TrackId, CommandWriter<ValueChangeCommand<Volume>>>,
}

impl TrackHandle {
	/// Returns the unique identifier for the mixer track.
	pub fn id(&self) -> TrackId {
		self.id
	}

	/// Sets the (post-effects) volume of the mixer track.
	pub fn set_volume(&mut self, volume: impl Into<Value<Volume>>, tween: Tween) {
		self.set_volume_command_writer.write(ValueChangeCommand {
			target: volume.into(),
			tween,
		})
	}

	/// Sets the volume of this track's route to another track.
	///
	/// This can only be used to change the volume of existing routes,
	/// not to add new routes.
	pub fn set_route(
		&mut self,
		to: impl Into<TrackId>,
		volume: impl Into<Value<Volume>>,
		tween: Tween,
	) -> Result<(), NonexistentRoute> {
		let to = to.into();
		self.route_set_volume_command_writers
			.get_mut(&to)
			.ok_or(NonexistentRoute)?
			.write(ValueChangeCommand {
				target: volume.into(),
				tween,
			});
		Ok(())
	}
}

impl Drop for TrackHandle {
	fn drop(&mut self) {
		if let Some(shared) = &self.shared {
			shared.mark_for_removal();
		}
	}
}
