use std::sync::Arc;

use crate::{command::ValueChangeCommand, Tween, Value};

use super::{CommandWriters, ListenerId, ListenerShared};

/// Controls a listener.
///
/// When a [`ListenerHandle`] is dropped, the corresponding
/// listener will be removed.
#[derive(Debug)]
pub struct ListenerHandle {
	pub(crate) id: ListenerId,
	pub(crate) shared: Arc<ListenerShared>,
	pub(crate) command_writers: CommandWriters,
}

impl ListenerHandle {
	/// Gets the unique identifier for this listener.
	pub fn id(&self) -> ListenerId {
		self.id
	}

	/// Sets the location of the listener in the spatial scene.
	pub fn set_position(&mut self, position: impl Into<Value<mint::Vector3<f32>>>, tween: Tween) {
		let position: Value<mint::Vector3<f32>> = position.into();
		self.command_writers.set_position.write(ValueChangeCommand {
			target: position.to_(),
			tween,
		})
	}

	/// Sets the delta time of the game loop. Needed for things like doppler on spatial tracks.
	pub fn set_game_loop_delta_time(&mut self, game_loop_delta_time: f64) {
		let game_loop_delta_time: Value<f64> = game_loop_delta_time.into();
		self.command_writers.set_game_loop_delta_time.write(ValueChangeCommand {
			target: game_loop_delta_time.to_(),
			tween: Tween::default(),
		})
	}

	/// Sets the rotation of the listener.
	///
	/// An unrotated listener should face in the negative Z direction with
	/// positive X to the right and positive Y up.
	pub fn set_orientation(
		&mut self,
		orientation: impl Into<Value<mint::Quaternion<f32>>>,
		tween: Tween,
	) {
		let orientation: Value<mint::Quaternion<f32>> = orientation.into();
		self.command_writers
			.set_orientation
			.write(ValueChangeCommand {
				target: orientation.to_(),
				tween,
			})
	}
}

impl Drop for ListenerHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}

impl From<&ListenerHandle> for ListenerId {
	fn from(value: &ListenerHandle) -> Self {
		value.id()
	}
}
