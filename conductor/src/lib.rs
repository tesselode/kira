use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Sample, Stream,
};
use std::{error::Error, f32::consts::PI};

pub struct AudioManager {
	stream: Stream,
}

impl AudioManager {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let host = cpal::default_host();
		let device = host.default_output_device().unwrap();
		let mut supported_configs_range = device.supported_output_configs().unwrap();
		let supported_config = supported_configs_range
			.next()
			.unwrap()
			.with_max_sample_rate();
		let config = supported_config.config();
		let sample_rate = config.sample_rate.0;
		let channels = config.channels;
		let mut phase = 0.0;
		let stream = device.build_output_stream(
			&config,
			move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
				for frame in data.chunks_exact_mut(channels as usize) {
					phase += 440.0 / (sample_rate as f32);
					phase %= 1.0;
					let out = 0.25 * (phase * 2.0 * PI).sin();
					for sample in frame.iter_mut() {
						*sample = Sample::from(&out);
					}
				}
			},
			move |err| {},
		)?;
		stream.play()?;
		Ok(Self { stream })
	}
}
