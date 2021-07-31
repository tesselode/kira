use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	BuildStreamError, DefaultStreamConfigError, Device, PlayStreamError, Stream, StreamConfig,
};
use kira::manager::{backend::Backend, renderer::Renderer, resources::UnusedResourceCollector};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceSetupError {
	/// A default audio output device could not be determined.
	#[error("Cannot find the default audio output device")]
	NoDefaultOutputDevice,

	/// An error occurred when getting the default output configuration.
	#[error("{0}")]
	DefaultStreamConfigError(#[from] DefaultStreamConfigError),
}

#[derive(Debug, Error)]
pub enum InitError {
	/// An error occured when building the audio stream.
	#[error("{0}")]
	BuildStreamError(#[from] BuildStreamError),

	/// An error occured when starting the audio stream.
	#[error("{0}")]
	PlayStreamError(#[from] PlayStreamError),
}

enum State {
	Uninitialized {
		device: Device,
		config: StreamConfig,
	},
	Initialized {
		_stream: Stream,
		unused_resource_collector: UnusedResourceCollector,
	},
}

pub struct CpalBackend {
	state: State,
}

impl CpalBackend {
	pub fn new() -> Result<Self, DeviceSetupError> {
		let host = cpal::default_host();
		let device = host
			.default_output_device()
			.ok_or(DeviceSetupError::NoDefaultOutputDevice)?;
		let config = device.default_output_config()?.config();
		Ok(Self {
			state: State::Uninitialized { device, config },
		})
	}

	pub fn free_unused_resources(&mut self) {
		match &mut self.state {
			State::Uninitialized { .. } => {
				panic!("Cannot free resources on an uninitialized backend")
			}
			State::Initialized {
				unused_resource_collector,
				..
			} => {
				unused_resource_collector.drain();
			}
		}
	}
}

impl Backend for CpalBackend {
	type InitError = InitError;

	fn sample_rate(&mut self) -> u32 {
		match &self.state {
			State::Uninitialized { config, .. } => config.sample_rate.0,
			State::Initialized { .. } => unreachable!(),
		}
	}

	fn init(
		&mut self,
		mut renderer: Renderer,
		unused_resource_collector: UnusedResourceCollector,
	) -> Result<(), Self::InitError> {
		match &mut self.state {
			State::Uninitialized { device, config } => {
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
				)?;
				stream.play()?;
				self.state = State::Initialized {
					_stream: stream,
					unused_resource_collector,
				};
				Ok(())
			}
			State::Initialized { .. } => panic!("Cannot initialize an already-initialized backend"),
		}
	}
}
