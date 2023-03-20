use ringbuf::HeapRb;

use crate::{parameter::Value, track::effect::EffectBuilder, Volume};

use super::{VolumeControl, VolumeControlHandle};

const COMMAND_CAPACITY: usize = 8;

/// Configures a volume control effect.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VolumeControlBuilder(pub Value<Volume>);

impl VolumeControlBuilder {
	/// Creates a new [`VolumeControlBuilder`].
	pub fn new(volume: impl Into<Value<Volume>>) -> Self {
		Self(volume.into())
	}
}

impl Default for VolumeControlBuilder {
	fn default() -> Self {
		Self(Value::Fixed(Volume::Amplitude(1.0)))
	}
}

impl EffectBuilder for VolumeControlBuilder {
	type Handle = VolumeControlHandle;

	fn build(self) -> (Box<dyn crate::track::effect::Effect>, Self::Handle) {
		let (command_producer, command_consumer) = HeapRb::new(COMMAND_CAPACITY).split();
		(
			Box::new(VolumeControl::new(self, command_consumer)),
			VolumeControlHandle { command_producer },
		)
	}
}
