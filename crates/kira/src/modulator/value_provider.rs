//! Contains types for reporting values of modulators.
//!
//! You'll only need these types if you're creating implementations
//! of the [`Sound`](crate::sound::Sound) or
//! [`Effect`](crate::track::effect::Effect) traits.

use atomic_arena::{error::ArenaFull, Arena};

use crate::manager::backend::resources::modulators::Modulators;

use super::ModulatorId;

/// Provides values of any modulator that currently exists.
pub struct ModulatorValueProvider<'a> {
	kind: ModulatorValueProviderKind<'a>,
}

impl<'a> ModulatorValueProvider<'a> {
	pub(crate) fn new(modulators: &'a Modulators) -> Self {
		Self {
			kind: ModulatorValueProviderKind::Normal { modulators },
		}
	}

	/// Gets the value of the modulator with the given ID if it
	/// exists, returns `None` otherwise.
	pub fn get(&self, id: ModulatorId) -> Option<f64> {
		match &self.kind {
			ModulatorValueProviderKind::Normal { modulators } => {
				modulators.get(id).map(|modulator| modulator.value())
			}
			ModulatorValueProviderKind::Mock {
				modulator_values: modulator_info,
			} => modulator_info.get(id.0).copied(),
		}
	}
}

enum ModulatorValueProviderKind<'a> {
	Normal { modulators: &'a Modulators },
	Mock { modulator_values: Arena<f64> },
}

/// Builds a `ModulatorValueProvider` that provides fake modulator value.
///
/// This is mainly useful for writing tests for implementations
/// of the [`Sound`](crate::sound::Sound) and
/// [`Effect`](crate::track::effect::Effect) traits.
pub struct MockModulatorValueProviderBuilder {
	modulator_values: Arena<f64>,
}

impl MockModulatorValueProviderBuilder {
	/// Creates a new [`MockModulatorValueProviderBuilder`] with room for
	/// the specified number of modulators.
	pub fn new(capacity: usize) -> Self {
		Self {
			modulator_values: Arena::new(capacity),
		}
	}

	/// Adds a new fake modulator to the builder and returns the corresponding
	/// [`ModulatorId`].
	pub fn add(&mut self, value: f64) -> Result<ModulatorId, ArenaFull> {
		Ok(ModulatorId(self.modulator_values.insert(value)?))
	}

	/// Consumes the builder and returns a [`ModulatorValueProvider`].
	pub fn build(self) -> ModulatorValueProvider<'static> {
		ModulatorValueProvider {
			kind: ModulatorValueProviderKind::Mock {
				modulator_values: self.modulator_values,
			},
		}
	}
}
