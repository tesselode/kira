pub mod static_sound;

use atomic_arena::Key;

use crate::{
	dsp::Frame,
	manager::resources::{Clocks, Parameters},
	track::TrackId,
};

/// A unique identifier for a parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundId(pub(crate) Key);

pub trait Sound: Send + Sync {
	fn sample_rate(&mut self) -> u32;

	fn track(&mut self) -> TrackId;

	fn on_start_processing(&mut self) {}

	fn process(&mut self, dt: f64, parameters: &Parameters, clocks: &Clocks) -> Frame;

	fn finished(&self) -> bool;
}
