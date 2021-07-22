pub mod backend;
pub mod error;

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};

use self::{backend::Backend, error::SetupError};

pub struct AudioManager {
	_stream: Stream,
}

impl AudioManager {
	pub fn new() -> Result<Self, SetupError> {
		let host = cpal::default_host();
		let device = host
			.default_output_device()
			.ok_or(SetupError::NoDefaultOutputDevice)?;
		let config = device.default_output_config()?.config();
		let sample_rate = config.sample_rate;
		let channels = config.channels;
		let mut backend = Backend::new(sample_rate.0);
		let stream = device.build_output_stream(
			&config,
			move |data: &mut [f32], _| {
				for frame in data.chunks_exact_mut(channels as usize) {
					let out = backend.process();
					if channels == 1 {
						frame[0] = (out.left + out.right) / 2.0;
					} else {
						frame[0] = out.left;
						frame[1] = out.right;
					}
				}
			},
			move |_| {},
		)?;
		stream.play()?;
		Ok(Self { _stream: stream })
	}
}
