use crate::{frame::Frame, manager::resources::Parameters};

#[allow(unused_variables)]
pub trait Effect: Send + Sync {
	fn init(&mut self, sample_rate: u32) {}

	fn process(&mut self, input: Frame, dt: f64, parameters: &Parameters) -> Frame;
}
