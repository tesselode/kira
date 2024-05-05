//! Contains types for reporting values of modulators.
//!
//! You'll only need these types if you're creating implementations
//! of the [`Sound`](crate::sound::Sound) or
//! [`Effect`](crate::effect::Effect) traits.

use crate::arena::{error::ArenaFull, Arena};

use super::{Modulator, ModulatorId};

/// Provides values of any modulator that currently exists.
pub struct ModulatorValueProvider<'a> {
	kind: ModulatorValueProviderKind<'a>,
}

impl<'a> ModulatorValueProvider<'a> {
	#[must_use]
	pub(crate) fn new(modulators: &'a Arena<Box<dyn Modulator>>) -> Self {
		Self {
			kind: ModulatorValueProviderKind::Normal { modulators },
		}
	}

	/// Gets the value of the modulator with the given ID if it
	/// exists, returns `None` otherwise.
	#[must_use]
	pub fn get(&self, id: ModulatorId) -> Option<f64> {
		match &self.kind {
			ModulatorValueProviderKind::Normal { modulators } => {
				modulators.get(id.0).map(|modulator| modulator.value())
			}
			ModulatorValueProviderKind::Mock {
				modulator_values: modulator_info,
			} => modulator_info.get(id.0).copied(),
		}
	}
}

enum ModulatorValueProviderKind<'a> {
	Normal {
		modulators: &'a Arena<Box<dyn Modulator>>,
	},
	Mock {
		modulator_values: Arena<f64>,
	},
}

/// Builds a `ModulatorValueProvider` that provides fake modulator values.
///
/// This is mainly useful for writing tests for implementations
/// of the [`Sound`](crate::sound::Sound) and
/// [`Effect`](crate::effect::Effect) traits.
pub struct MockModulatorValueProviderBuilder {
	modulator_values: Arena<f64>,
}

impl MockModulatorValueProviderBuilder {
	/// Creates a new [`MockModulatorValueProviderBuilder`] with room for
	/// the specified number of modulators.
	#[must_use]
	pub fn new(capacity: u16) -> Self {
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
	#[must_use]
	pub fn build(self) -> ModulatorValueProvider<'static> {
		ModulatorValueProvider {
			kind: ModulatorValueProviderKind::Mock {
				modulator_values: self.modulator_values,
			},
		}
	}
}
