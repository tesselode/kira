use crate::manager::backend::{Backend, Renderer};
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Device, Stream, StreamConfig,
};

use super::{CpalBackendSettings, Error};

enum State {
	Empty,
	Uninitialized {
		device: Device,
		config: StreamConfig,
	},
	Initialized {
		_stream: Stream,
	},
}

/// A backend that uses [cpal](https://crates.io/crates/cpal) to
/// connect a [`Renderer`] to the operating system's audio driver.
pub struct CpalBackend {
	state: State,
}

impl Backend for CpalBackend {
	type Settings = CpalBackendSettings;

	type Error = Error;

	fn setup(settings: Self::Settings) -> Result<(Self, u32), Self::Error> {
		let host = cpal::default_host();
		let device = if let Some(device) = settings.device {
			device
		} else {
			host.default_output_device()
				.ok_or(Error::NoDefaultOutputDevice)?
		};
		let config = device.default_output_config()?.config();
		let sample_rate = config.sample_rate.0;
		Ok((
			Self {
				state: State::Uninitialized { device, config },
			},
			sample_rate,
		))
	}

	fn start(&mut self, mut renderer: Renderer) -> Result<(), Self::Error> {
		if let State::Uninitialized { device, config } =
			std::mem::replace(&mut self.state, State::Empty)
		{
			let channels = config.channels;
			let stream = device.build_output_stream(
				&config,
				move |data: &mut [f32], _| {
					renderer.on_start_processing();
					for frame in data.chunks_exact_mut(channels as usize) {
						let out = renderer.process();
						if channels == 1 {
							frame[0] = (out.left + out.right) / 2.0;
						} else {
							frame[0] = out.left;
							frame[1] = out.right;
						}
					}
				},
				move |_| {},
				None,
			)?;
			stream.play()?;
			self.state = State::Initialized { _stream: stream };
		} else {
			panic!("Cannot initialize the backend multiple times")
		}
		Ok(())
	}
}
