use crate::{
	manager::resources::{Clocks, Parameters},
	track::TrackId,
	Frame,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessResult {
	Loaded(Frame),
	Waiting,
}

pub trait Sound: Send + Sync {
	fn sample_rate(&mut self) -> u32;

	fn track(&mut self) -> TrackId;

	fn on_start_processing(&mut self) {}

	fn process(&mut self, parameters: &Parameters, clocks: &Clocks) -> ProcessResult;

	fn finished(&mut self) -> bool;
}
