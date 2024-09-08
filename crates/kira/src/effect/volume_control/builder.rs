use crate::{effect::EffectBuilder, tween::Value, Dbfs};

use super::{command_writers_and_readers, VolumeControl, VolumeControlHandle};

/// Configures a volume control effect.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VolumeControlBuilder(pub Value<Dbfs>);

impl VolumeControlBuilder {
	/// Creates a new [`VolumeControlBuilder`].
	#[must_use]
	pub fn new(volume: impl Into<Value<Dbfs>>) -> Self {
		Self(volume.into())
	}
}

impl Default for VolumeControlBuilder {
	fn default() -> Self {
		Self(Value::Fixed(Dbfs::IDENTITY))
	}
}

impl EffectBuilder for VolumeControlBuilder {
	type Handle = VolumeControlHandle;

	fn build(self) -> (Box<dyn crate::effect::Effect>, Self::Handle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		(
			Box::new(VolumeControl::new(self, command_readers)),
			VolumeControlHandle { command_writers },
		)
	}
}
