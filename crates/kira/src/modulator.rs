pub mod lfo;
pub mod tweener;
pub mod value_provider;

use atomic_arena::Key;

use crate::clock::clock_info::ClockInfoProvider;

use self::value_provider::ModulatorValueProvider;

/// Configures a modulator.
pub trait ModulatorBuilder {
	/// Allows the user to control the modulator from gameplay code.
	type Handle;

	/// Creates the modulator and a handle to the modulator.
	fn build(self, id: ModulatorId) -> (Box<dyn Modulator>, Self::Handle);
}

pub trait Modulator: Send {
	/// Called whenever a new batch of audio samples is requested by the backend.
	///
	/// This is a good place to put code that needs to run fairly frequently,
	/// but not for every single audio sample.
	fn on_start_processing(&mut self) {}

	fn update(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	);

	fn value(&self) -> f64;

	/// Whether the modulator can be removed from the audio context.
	fn finished(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModulatorId(pub(crate) Key);
