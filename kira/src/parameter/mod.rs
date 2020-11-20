//! Provides an interface for smoothly changing values over time.

mod mapping;
mod parameter;
mod parameters;
mod tween;

pub use mapping::Mapping;
pub use parameter::{Parameter, ParameterId};
pub use parameters::Parameters;
pub use tween::Tween;
