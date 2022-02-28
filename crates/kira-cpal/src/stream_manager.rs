mod renderer_wrapper;

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
use kira::manager::backend::Renderer;
use ringbuf::{Consumer, RingBuffer};

use crate::Error;

use self::renderer_wrapper::RendererWrapper;

const CHECK_STREAM_INTERVAL: Duration = Duration::from_millis(500);

#[allow(clippy::large_enum_variant)]
enum State {
	Empty,
	Idle {
		renderer: Renderer,
	},
	Running {
		stream: Stream,
		stream_error_consumer: Consumer<StreamError>,
		renderer_consumer: Consumer<Renderer>,
	},
}

pub(super) struct StreamManagerController {
	should_drop: Arc<AtomicBool>,
}

impl StreamManagerController {
	pub fn stop(&self) {
		self.should_drop.store(true, Ordering::SeqCst);
	}
}

/// Starts a cpal stream and restarts it if needed
/// in the case of device changes or disconnections.
pub(super) struct StreamManager {
	state: State,
}

impl StreamManager {
	pub fn start(
		renderer: Renderer,
		device: Device,
		config: StreamConfig,
	) -> StreamManagerController {
		let should_drop = Arc::new(AtomicBool::new(false));
		let should_drop_clone = should_drop.clone();
		std::thread::spawn(move || {
			let mut stream_manager = StreamManager {
				state: State::Idle { renderer },
			};
			stream_manager.start_stream(&device, &config).unwrap();
			loop {
				std::thread::sleep(CHECK_STREAM_INTERVAL);
				if should_drop.load(Ordering::SeqCst) {
					break;
				}
				stream_manager.check_stream();
			}
		});
		StreamManagerController {
			should_drop: should_drop_clone,
		}
	}

	/// Restarts the stream if the audio device gets disconnected.
	fn check_stream(&mut self) {
		if let State::Running {
			stream_error_consumer,
			..
		} = &mut self.state
		{
			if let Some(StreamError::DeviceNotAvailable) = stream_error_consumer.pop() {
				self.stop_stream();
				if let Ok((device, config)) = default_device_and_config() {
					// TODO: gracefully handle errors that occur in this function
					self.start_stream(&device, &config).unwrap();
				}
			}
		}
	}

	fn start_stream(&mut self, device: &Device, config: &StreamConfig) -> Result<(), Error> {
		let mut renderer =
			if let State::Idle { renderer } = std::mem::replace(&mut self.state, State::Empty) {
				renderer
			} else {
				panic!("trying to start a stream when the stream manager is not idle");
			};
		renderer.on_change_sample_rate(config.sample_rate.0);
		let (mut renderer_wrapper, renderer_consumer) = RendererWrapper::new(renderer);
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
		self.state = State::Running {
			stream,
			stream_error_consumer,
			renderer_consumer,
		};
		Ok(())
	}

	fn stop_stream(&mut self) {
		if let State::Running {
			mut renderer_consumer,
			stream,
			..
		} = std::mem::replace(&mut self.state, State::Empty)
		{
			drop(stream);
			let renderer = renderer_consumer
				.pop()
				.expect("Could not retrieve the renderer after dropping a stream");
			self.state = State::Idle { renderer };
		} else {
			panic!("Trying to stop the stream when it's not running")
		}
	}
}

fn default_device_and_config() -> Result<(Device, StreamConfig), Error> {
	let host = cpal::default_host();
	let device = host
		.default_output_device()
		.ok_or(Error::NoDefaultOutputDevice)?;
	let config = device.default_output_config()?.config();
	Ok((device, config))
}
