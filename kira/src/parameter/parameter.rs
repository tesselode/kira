use uuid::Uuid;

use crate::util::generate_uuid;

use super::Tween;

/**
A unique identifier for a parameter.

You cannot create this manually - a parameter ID is created
when you create a parameter with an [`AudioManager`](crate::manager::AudioManager).
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
pub struct ParameterId {
	uuid: Uuid,
}

impl ParameterId {
	pub(crate) fn new() -> Self {
		Self {
			uuid: generate_uuid(),
		}
	}
}

/// Settings for a parameter.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct ParameterSettings {
	/// The unique identifier for the parameter.
	pub id: ParameterId,
	/// The volume of the parameter.
	pub value: f64,
}

impl ParameterSettings {
	/// Creates a new `ParameterSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the unique identifier for the parameter.
	pub fn id(self, id: impl Into<ParameterId>) -> Self {
		Self {
			id: id.into(),
			..self
		}
	}

	/// Sets the initial value of the parameter.
	pub fn value(self, value: impl Into<f64>) -> Self {
		Self {
			value: value.into(),
			..self
		}
	}
}

impl Default for ParameterSettings {
	fn default() -> Self {
		Self {
			id: ParameterId::new(),
			value: 1.0,
		}
	}
}

#[derive(Debug, Copy, Clone)]
struct TweenState {
	tween: Tween,
	start: f64,
	end: f64,
	time: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct Parameter {
	value: f64,
	tween_state: Option<TweenState>,
}

impl Parameter {
	pub(crate) fn new(value: f64) -> Self {
		Self {
			value,
			tween_state: None,
		}
	}

	pub(crate) fn value(&self) -> f64 {
		self.value
	}

	pub(crate) fn set(&mut self, target: f64, tween: Option<Tween>) {
		if let Some(tween) = tween {
			self.tween_state = Some(TweenState {
				tween,
				start: self.value,
				end: target,
				time: 0.0,
			});
		} else {
			self.value = target;
		}
	}

	pub(crate) fn update(&mut self, dt: f64) -> bool {
		if let Some(tween_state) = &mut self.tween_state {
			tween_state.time += dt;
			self.value =
				tween_state
					.tween
					.tween(tween_state.start, tween_state.end, tween_state.time);
			if tween_state.time >= tween_state.tween.duration {
				self.tween_state = None;
				return true;
			}
		}
		false
	}
}
