/*!
# kira-cpal

kira-cpal is a [Kira](https://crates.io/crates/kira) backend
for desktop targets.

## Examples

### Setting up an `AudioManager` with a `CpalBackend`

```no_run
use kira::manager::{AudioManager, AudioManagerSettings};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::<CpalBackend>::new(
	AudioManagerSettings::default(),
)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```
*/

#![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

mod error;
mod renderer_wrapper;

pub use error::*;
use renderer_wrapper::RendererWrapper;

use std::{
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Device, Stream, StreamConfig, StreamError,
};
use kira::manager::backend::{Backend, Renderer};
use ringbuf::{Consumer, Producer, RingBuffer};

const RESULT_POLLING_INTERVAL: Duration = Duration::from_millis(1);
const DEVICE_POLLING_INTERVAL: Duration = Duration::from_millis(500);

enum State {
	Empty,
	Uninitialized {
		device: Device,
		config: StreamConfig,
	},
	Initialized {
		should_stop: Arc<AtomicBool>,
	},
}

pub struct CpalBackend {
	state: State,
}

impl Backend for CpalBackend {
	type Settings = ();

	type Error = Error;

	fn setup(_settings: Self::Settings) -> Result<(Self, u32), Self::Error> {
		let host = cpal::default_host();
		let device = host
			.default_output_device()
			.ok_or(Error::NoDefaultOutputDevice)?;
		let config = device.default_output_config()?.config();
		let sample_rate = config.sample_rate.0;
		Ok((
			Self {
				state: State::Uninitialized { device, config },
			},
			sample_rate,
		))
	}

	fn start(&mut self, renderer: Renderer) -> Result<(), Self::Error> {
		let state = std::mem::replace(&mut self.state, State::Empty);
		if let State::Uninitialized { device, config } = state {
			let (setup_result_producer, mut setup_result_consumer) = RingBuffer::new(1).split();
			let should_stop = Arc::new(AtomicBool::new(false));
			let should_stop_clone = should_stop.clone();
			std::thread::spawn(move || {
				manage_streams(
					device,
					config,
					renderer,
					setup_result_producer,
					should_stop_clone,
				);
			});
			self.state = State::Initialized { should_stop };
			loop {
				std::thread::sleep(RESULT_POLLING_INTERVAL);
				if let Some(result) = setup_result_consumer.pop() {
					return result;
				}
			}
		} else {
			panic!("Cannot initialize the backend multiple times")
		}
	}
}

impl Drop for CpalBackend {
	fn drop(&mut self) {
		if let State::Initialized { should_stop } = &self.state {
			should_stop.store(true, Ordering::SeqCst);
		}
	}
}

fn manage_streams(
	device: Device,
	config: StreamConfig,
	renderer: Renderer,
	mut setup_result_producer: Producer<Result<(), Error>>,
	should_stop: Arc<AtomicBool>,
) {
	let (renderer_producer, mut renderer_consumer) = RingBuffer::new(1).split();
	let renderer_wrapper = RendererWrapper {
		renderer: Some(renderer),
		producer: renderer_producer,
	};
	let (mut stream, mut stream_error_consumer) =
		match start_stream(&device, &config, renderer_wrapper) {
			Ok(stream) => {
				setup_result_producer
					.push(Ok(()))
					.expect("Setup result producer is full");
				stream
			}
			Err(error) => {
				setup_result_producer
					.push(Err(error))
					.expect("Setup result producer is full");
				return;
			}
		};
	loop {
		std::thread::sleep(DEVICE_POLLING_INTERVAL);
		if should_stop.load(Ordering::SeqCst) {
			break;
		}
		if let Some(StreamError::DeviceNotAvailable) = stream_error_consumer.pop() {
			let (new_stream, new_stream_error_consumer, new_renderer_consumer) =
				restart_stream(stream, renderer_consumer).unwrap();
			stream = new_stream;
			stream_error_consumer = new_stream_error_consumer;
			renderer_consumer = new_renderer_consumer;
		}
	}
}

fn start_stream(
	device: &Device,
	config: &StreamConfig,
	mut renderer_wrapper: RendererWrapper,
) -> Result<(Stream, Consumer<StreamError>), Error> {
	let (mut stream_error_producer, stream_error_consumer) = RingBuffer::new(1).split();
	let channels = config.channels;
	let stream = device.build_output_stream(
		config,
		move |data: &mut [f32], _| {
			renderer_wrapper.on_start_processing();
			for frame in data.chunks_exact_mut(channels as usize) {
				let out = renderer_wrapper.process();
				if channels == 1 {
					frame[0] = (out.left + out.right) / 2.0;
				} else {
					frame[0] = out.left;
					frame[1] = out.right;
				}
			}
		},
		move |error| {
			stream_error_producer
				.push(error)
				.expect("Stream error producer is full");
		},
	)?;
	stream.play()?;
	Ok((stream, stream_error_consumer))
}

fn restart_stream(
	stream: Stream,
	mut renderer_consumer: Consumer<Renderer>,
) -> Result<(Stream, Consumer<StreamError>, Consumer<Renderer>), Error> {
	drop(stream);
	let mut renderer = renderer_consumer
		.pop()
		.expect("Could not retrieve the renderer after dropping a stream");
	let host = cpal::default_host();
	let device = host
		.default_output_device()
		.ok_or(Error::NoDefaultOutputDevice)?;
	let config = device.default_output_config()?.config();
	renderer.on_change_sample_rate(config.sample_rate.0);
	let (renderer_producer, renderer_consumer) = RingBuffer::new(1).split();
	let renderer_wrapper = RendererWrapper {
		renderer: Some(renderer),
		producer: renderer_producer,
	};
	let (stream, stream_error_consumer) = start_stream(&device, &config, renderer_wrapper)?;
	Ok((stream, stream_error_consumer, renderer_consumer))
}
