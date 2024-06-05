use std::{collections::HashMap, error::Error, fmt::Display, sync::Arc};

use crate::{
	command::{CommandWriter, ValueChangeCommand},
	tween::{Tween, Value},
	Trigger, Volume,
};

use super::{TrackId, TrackShared};

/// An error that's returned when trying to change the volume of a track route
/// that did not exist originally.
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
#[derive(Debug)]
pub struct TrackHandle {
	pub(crate) id: TrackId,
	pub(crate) shared: Option<Arc<TrackShared>>,
	pub(crate) set_volume_command_writer: CommandWriter<ValueChangeCommand<Volume>>,
	pub(crate) route_set_volume_command_writers:
		HashMap<TrackId, CommandWriter<ValueChangeCommand<Volume>>>,
}

impl TrackHandle {
	/// Returns the unique identifier for the mixer track.
	#[must_use]
	pub fn id(&self) -> TrackId {
		self.id
	}

	/// Sets the (post-effects) volume of the mixer track.
	pub fn set_volume(&mut self, volume: impl Into<Value<Volume>>, tween: Tween) -> Trigger {
		let trigger = Trigger::new();
		self.set_volume_command_writer.write(ValueChangeCommand {
			target: volume.into(),
			tween,
			finish_trigger: Some(trigger.clone()),
		});
		trigger
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
	) -> Result<Trigger, NonexistentRoute> {
		let trigger = Trigger::new();
		let to = to.into();
		self.route_set_volume_command_writers
			.get_mut(&to)
			.ok_or(NonexistentRoute)?
			.write(ValueChangeCommand {
				target: volume.into(),
				tween,
				finish_trigger: Some(trigger.clone()),
			});
		Ok(trigger)
	}
}

impl Drop for TrackHandle {
	fn drop(&mut self) {
		if let Some(shared) = &self.shared {
			shared.mark_for_removal();
		}
	}
}
