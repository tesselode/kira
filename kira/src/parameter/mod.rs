//! Tweenable values that can be used by many other objects.

pub mod handle;
mod mapping;
mod parameter;
mod parameters;
mod tween;

pub use mapping::Mapping;
pub(crate) use parameter::Parameter;
pub use parameter::{ParameterId, ParameterSettings};
pub use parameters::Parameters;
pub use tween::{EaseDirection, Easing, Tween};
