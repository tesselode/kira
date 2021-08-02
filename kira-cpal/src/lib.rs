use std::time::Duration;

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	BuildStreamError, DefaultStreamConfigError, Device, PlayStreamError, Stream, StreamConfig,
};
use kira::manager::{renderer::Renderer, resources::UnusedResourceCollector, Backend};
use ringbuf::{Producer, RingBuffer};
use thiserror::Error;

const UNUSED_RESOURCE_COLLECTION_INTERVAL: Duration = Duration::from_millis(100);

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
		quit_signal_producer: Producer<()>,
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
		renderer: Renderer,
		unused_resource_collector: UnusedResourceCollector,
	) -> Result<(), Self::InitError> {
		match &mut self.state {
			State::Uninitialized { device, config } => {
				let stream = setup_stream(config, device, renderer)?;
				let quit_signal_producer =
					start_resource_collection_thread(unused_resource_collector);
				self.state = State::Initialized {
					_stream: stream,
					quit_signal_producer,
				};
				Ok(())
			}
			State::Initialized { .. } => panic!("Cannot initialize an already-initialized backend"),
		}
	}
}

impl Drop for CpalBackend {
	fn drop(&mut self) {
		if let State::Initialized {
			quit_signal_producer,
			..
		} = &mut self.state
		{
			if quit_signal_producer.push(()).is_err() {
				panic!("Could not send the quit signal to end the resource collection thread");
			}
		}
	}
}

fn setup_stream(
	config: &mut StreamConfig,
	device: &mut Device,
	mut renderer: Renderer,
) -> Result<Stream, InitError> {
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
	Ok(stream)
}

fn start_resource_collection_thread(
	mut unused_resource_collector: UnusedResourceCollector,
) -> Producer<()> {
	let (quit_signal_producer, mut quit_signal_consumer) = RingBuffer::new(1).split();
	std::thread::spawn(move || {
		while quit_signal_consumer.pop().is_none() {
			std::thread::sleep(UNUSED_RESOURCE_COLLECTION_INTERVAL);
			unused_resource_collector.drain();
		}
	});
	quit_signal_producer
}
