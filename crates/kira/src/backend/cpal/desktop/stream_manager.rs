mod send_on_drop;

use std::{
	sync::{
		atomic::{AtomicBool, AtomicU64, Ordering},
		Arc,
	},
	time::Duration,
};

use super::renderer_with_cpu_usage::RendererWithCpuUsage;
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	BufferSize, Device, Stream, StreamConfig, StreamError,
};
use rtrb::{Consumer, PushError, RingBuffer};
use send_on_drop::SendOnDrop;

use super::super::Error;

const CHECK_STREAM_INTERVAL: Duration = Duration::from_millis(500);

#[allow(clippy::large_enum_variant)]
enum State {
	Empty,
	Idle {
		renderer: RendererWithCpuUsage,
	},
	Running {
		stream: Stream,
		_stream_errors_discarded: Arc<AtomicU64>,
		renderer_consumer: Consumer<RendererWithCpuUsage>,
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
	device_name: String,
	sample_rate: u32,
	custom_device: bool,
	buffer_size: BufferSize,
}

impl StreamManager {
	pub fn start(
		renderer: RendererWithCpuUsage,
		device: Device,
		mut config: StreamConfig,
		custom_device: bool,
		buffer_size: BufferSize,
	) -> StreamManagerController {
		let should_drop = Arc::new(AtomicBool::new(false));
		let should_drop_clone = should_drop.clone();
		std::thread::spawn(move || {
			let mut stream_manager = StreamManager {
				state: State::Idle { renderer },
				device_name: device_name(&device),
				sample_rate: config.sample_rate.0,
				custom_device,
				buffer_size,
			};
			let mut stream_error_consumer =
				stream_manager.start_stream(&device, &mut config).unwrap();
			loop {
				std::thread::sleep(CHECK_STREAM_INTERVAL);
				if should_drop.load(Ordering::SeqCst) {
					break;
				}
				stream_manager.check_stream(&mut stream_error_consumer);
			}
		});
		StreamManagerController {
			should_drop: should_drop_clone,
		}
	}

	/// Restarts the stream if the audio device gets disconnected.
	fn check_stream(&mut self, stream_error_consumer: &mut Consumer<StreamError>) {
		if let State::Running { .. } = &self.state {
			while let Ok(error) = stream_error_consumer.pop() {
				match error {
					// check for device disconnection
					StreamError::DeviceNotAvailable => {
						self.stop_stream();
						if let Ok((device, mut config)) = default_device_and_config() {
							// TODO: gracefully handle errors that occur in this function
							self.start_stream(&device, &mut config).unwrap();
						}
					}
					StreamError::BackendSpecific { err: _ } => {}
				}
			}
			// check for device changes if a custom device hasn't been specified
			// Disabled on macos due to audio artifacts that seem to occur when the device is
			// queried while playing.
			// see: https://github.com/tesselode/kira/issues/38
			#[cfg(not(target_os = "macos"))]
			if !self.custom_device {
				if let Ok((device, mut config)) = default_device_and_config() {
					let device_name = device_name(&device);
					let sample_rate = config.sample_rate.0;
					if device_name != self.device_name || sample_rate != self.sample_rate {
						self.stop_stream();
						self.start_stream(&device, &mut config).unwrap();
					}
				}
			}
		}
	}

	fn start_stream(
		&mut self,
		device: &Device,
		config: &mut StreamConfig,
	) -> Result<Consumer<StreamError>, Error> {
		let mut renderer =
			if let State::Idle { renderer } = std::mem::replace(&mut self.state, State::Empty) {
				renderer
			} else {
				panic!("trying to start a stream when the stream manager is not idle");
			};
		config.buffer_size = self.buffer_size; // this won't change anything if the buffer size is BufferSize::Default
		let device_name = device_name(device);
		let sample_rate = config.sample_rate.0;
		if sample_rate != self.sample_rate {
			renderer.on_change_sample_rate(sample_rate);
		}
		self.device_name = device_name;
		self.sample_rate = sample_rate;
		let (mut renderer_wrapper, renderer_consumer) = SendOnDrop::new(renderer);
		let (mut stream_error_producer, stream_error_consumer) = RingBuffer::new(64);
		let stream_errors_discarded = Arc::new(AtomicU64::new(0));
		let stream_errors_discarded_clone = Arc::clone(&stream_errors_discarded);
		let channels = config.channels;
		let stream = device.build_output_stream(
			config,
			move |data: &mut [f32], _| {
				#[cfg(feature = "assert_no_alloc")]
				assert_no_alloc::assert_no_alloc(|| {
					process_renderer(&mut renderer_wrapper, data, channels, sample_rate);
				});
				#[cfg(not(feature = "assert_no_alloc"))]
				process_renderer(&mut renderer_wrapper, data, channels, sample_rate);
			},
			move |error| match stream_error_producer.push(error) {
				Ok(()) => {}
				Err(PushError::Full(_stream_error)) => {
					stream_errors_discarded.fetch_add(1, Ordering::AcqRel);
				}
			},
			None,
		)?;
		stream.play()?;
		self.state = State::Running {
			stream,
			_stream_errors_discarded: stream_errors_discarded_clone,
			renderer_consumer,
		};
		Ok(stream_error_consumer)
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

fn device_name(device: &Device) -> String {
	device
		.name()
		.unwrap_or_else(|_| "device name unavailable".to_string())
}

fn process_renderer(
	renderer: &mut SendOnDrop<RendererWithCpuUsage>,
	data: &mut [f32],
	channels: u16,
	sample_rate: u32,
) {
	renderer.on_start_processing();
	renderer.process(data, channels, sample_rate);
}
