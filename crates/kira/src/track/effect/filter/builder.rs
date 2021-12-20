use ringbuf::RingBuffer;

use crate::track::effect::{Effect, EffectBuilder};

use super::{Filter, FilterHandle, FilterMode};

const COMMAND_CAPACITY: usize = 8;

/// Configures a [`Filter`].
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub struct FilterBuilder {
	/// The frequencies that the filter will remove.
	pub mode: FilterMode,
	/// The cutoff frequency of the filter (in hertz).
	pub cutoff: f64,
	/// The resonance of the filter.
	///
	/// The resonance is a feedback effect that produces
	/// a distinctive "ringing" sound.
	pub resonance: f64,
	/// How much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means
	/// only the dry signal will be heard. `1.0` means
	/// only the wet signal will be heard.
	pub mix: f64,
}

impl FilterBuilder {
	/// Creates a new `FilterBuilder` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the frequencies that the filter will remove.
	pub fn mode(self, mode: FilterMode) -> Self {
		Self { mode, ..self }
	}

	/// Sets the cutoff frequency of the filter (in hertz).
	pub fn cutoff(self, cutoff: f64) -> Self {
		Self { cutoff, ..self }
	}

	/// Sets the resonance of the filter.
	pub fn resonance(self, resonance: f64) -> Self {
		Self { resonance, ..self }
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn mix(self, mix: f64) -> Self {
		Self { mix, ..self }
	}
}

impl Default for FilterBuilder {
	fn default() -> Self {
		Self {
			mode: FilterMode::LowPass,
			cutoff: 1000.0,
			resonance: 0.0,
			mix: 1.0,
		}
	}
}

impl EffectBuilder for FilterBuilder {
	type Handle = FilterHandle;

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		let (command_producer, command_consumer) = RingBuffer::new(COMMAND_CAPACITY).split();
		(
			Box::new(Filter::new(self, command_consumer)),
			FilterHandle { command_producer },
		)
	}
}
