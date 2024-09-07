use std::sync::Arc;

use crate::{
	command::{CommandWriter, ValueChangeCommand},
	tween::{Tween, Value},
	Volume,
};

use super::{SendTrackId, TrackShared};

/// Controls a mixer track.
///
/// When a [`SendTrackHandle`] is dropped, the corresponding mixer
/// track will be removed.
#[derive(Debug)]
pub struct SendTrackHandle {
	pub(crate) id: SendTrackId,
	pub(crate) shared: Option<Arc<TrackShared>>,
	pub(crate) set_volume_command_writer: CommandWriter<ValueChangeCommand<Volume>>,
}

impl SendTrackHandle {
	/// Returns a unique identifier for this send track.
	pub fn id(&self) -> SendTrackId {
		self.id
	}

	/// Sets the (post-effects) volume of the send track.
	pub fn set_volume(&mut self, volume: impl Into<Value<Volume>>, tween: Tween) {
		self.set_volume_command_writer.write(ValueChangeCommand {
			target: volume.into(),
			tween,
		})
	}
}

impl Drop for SendTrackHandle {
	fn drop(&mut self) {
		if let Some(shared) = &self.shared {
			shared.mark_for_removal();
		}
	}
}
