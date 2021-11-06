pub mod static_sound;

use crate::{
	dsp::Frame,
	manager::resources::{Clocks, Parameters},
	track::TrackId,
};

pub trait SoundData {
	type Handle;

	fn into_sound(self) -> (Box<dyn Sound>, Self::Handle);
}

pub trait Sound: Send + Sync {
	fn track(&mut self) -> TrackId;

	fn on_start_processing(&mut self) {}

	fn process(&mut self, dt: f64, parameters: &Parameters, clocks: &Clocks) -> Frame;

	fn finished(&self) -> bool;
}
