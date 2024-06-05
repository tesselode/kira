use std::sync::Arc;

use crate::{
	command::ValueChangeCommand,
	tween::{Tween, Value},
	Trigger,
};

use super::{CommandWriters, ListenerShared};

/// Controls a listener.
///
/// When a [`ListenerHandle`] is dropped, the corresponding
/// listener will be removed.
#[derive(Debug)]
pub struct ListenerHandle {
	pub(crate) shared: Arc<ListenerShared>,
	pub(crate) command_writers: CommandWriters,
}

impl ListenerHandle {
	/// Sets the location of the listener in the spatial scene.
	pub fn set_position(
		&mut self,
		position: impl Into<Value<mint::Vector3<f32>>>,
		tween: Tween,
	) -> Trigger {
		let finish_trigger = Trigger::new();
		let position: Value<mint::Vector3<f32>> = position.into();
		self.command_writers.set_position.write(ValueChangeCommand {
			target: position.to_(),
			tween,
			finish_trigger: Some(finish_trigger.clone()),
		});
		finish_trigger
	}

	/// Sets the rotation of the listener.
	///
	/// An unrotated listener should face in the negative Z direction with
	/// positive X to the right and positive Y up.
	pub fn set_orientation(
		&mut self,
		orientation: impl Into<Value<mint::Quaternion<f32>>>,
		tween: Tween,
	) -> Trigger {
		let finish_trigger = Trigger::new();
		let orientation: Value<mint::Quaternion<f32>> = orientation.into();
		self.command_writers
			.set_orientation
			.write(ValueChangeCommand {
				target: orientation.to_(),
				tween,
				finish_trigger: Some(finish_trigger.clone()),
			});
		finish_trigger
	}
}

impl Drop for ListenerHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}
