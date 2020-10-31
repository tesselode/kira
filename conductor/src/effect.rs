use crate::stereo_sample::StereoSample;

pub trait Effect {
	fn process(&mut self, dt: f64, input: StereoSample) -> StereoSample;
}
