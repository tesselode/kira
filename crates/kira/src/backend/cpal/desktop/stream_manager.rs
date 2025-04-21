mod send_on_drop;

use std::{
	sync::{
		atomic::{AtomicBool, AtomicU64, Ordering},
		Arc, Mutex,
	},
	time::Duration,
};

use super::renderer_with_cpu_usage::RendererWithCpuUsage;
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	BufferSize, Device, Stream, StreamConfig, StreamError,
};
use rtrb::{Consumer, Producer, PushError, RingBuffer};
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
		renderer_consumer: Consumer<RendererWithCpuUsage>,
	},
}

pub(super) struct StreamManagerController {
	should_drop: Arc<AtomicBool>,
	num_stream_errors_discarded: Arc<AtomicU64>,
	handled_stream_error_consumer: Mutex<Consumer<StreamError>>,
}

impl StreamManagerController {
	pub fn stop(&self) {
		self.should_drop.store(true, Ordering::SeqCst);
	}

	#[must_use]
	pub fn num_stream_errors_discarded(&self) -> u64 {
		self.num_stream_errors_discarded.load(Ordering::Acquire)
	}

	pub fn pop_handled_error(&mut self) -> Option<StreamError> {
		self.handled_stream_error_consumer
			.get_mut()
			.unwrap()
			.pop()
			.ok()
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
	) -> Result<StreamManagerController, Error> {
		let should_drop = Arc::new(AtomicBool::new(false));
		let should_drop_clone = should_drop.clone();

		let num_stream_errors_discarded = Arc::new(AtomicU64::new(0));
		let num_stream_errors_discarded_clone = num_stream_errors_discarded.clone();

		let (mut handled_stream_error_producer, handled_stream_error_consumer) =
			RingBuffer::new(64);

		let (mut initial_result_producer, mut initial_result_consumer) = RingBuffer::new(1);

		std::thread::spawn(move || {
			let mut stream_manager = StreamManager {
				state: State::Idle { renderer },
				device_name: device_name(&device),
				sample_rate: config.sample_rate.0,
				custom_device,
				buffer_size,
			};
			let mut unhandled_stream_error_consumer = match stream_manager.start_stream(
				&device,
				&mut config,
				num_stream_errors_discarded.clone(),
			) {
				Ok(unhandled_stream_error_consumer) => {
					initial_result_producer.push(Ok(())).unwrap();
					unhandled_stream_error_consumer
				}
				Err(err) => {
					initial_result_producer.push(Err(err)).unwrap();
					return;
				}
			};
			loop {
				std::thread::sleep(CHECK_STREAM_INTERVAL);
				if should_drop.load(Ordering::SeqCst) {
					break;
				}
				stream_manager.check_stream(
					&mut unhandled_stream_error_consumer,
					&mut handled_stream_error_producer,
					&num_stream_errors_discarded,
				);
			}
		});

		loop {
			if let Ok(result) = initial_result_consumer.pop() {
				result?;
				break;
			}
			std::thread::sleep(Duration::from_micros(100));
		}

		Ok(StreamManagerController {
			should_drop: should_drop_clone,
			num_stream_errors_discarded: num_stream_errors_discarded_clone,
			handled_stream_error_consumer: Mutex::new(handled_stream_error_consumer),
		})
	}

	/// Restarts the stream if the audio device gets disconnected.
	fn check_stream(
		&mut self,
		unhandled_stream_error_consumer: &mut Consumer<StreamError>,
		handled_stream_error_producer: &mut Producer<StreamError>,
		num_stream_errors_discarded: &Arc<AtomicU64>,
	) {
		if let State::Running { .. } = &self.state {
			while let Ok(error) = unhandled_stream_error_consumer.pop() {
				match error {
					// check for device disconnection
					StreamError::DeviceNotAvailable => {
						self.stop_stream();
						if let Ok((device, mut config)) = default_device_and_config() {
							// TODO: gracefully handle errors that occur in this function
							*unhandled_stream_error_consumer = self
								.start_stream(
									&device,
									&mut config,
									num_stream_errors_discarded.clone(),
								)
								.unwrap();
						}
					}
					StreamError::BackendSpecific { err: _ } => {}
				}
				match handled_stream_error_producer.push(error) {
					Ok(()) => {}
					Err(PushError::Full(_stream_error)) => {
						num_stream_errors_discarded.fetch_add(1, Ordering::AcqRel);
					}
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
						*unhandled_stream_error_consumer = self
							.start_stream(&device, &mut config, num_stream_errors_discarded.clone())
							.unwrap();
					}
				}
			}
		}
	}

	fn start_stream(
		&mut self,
		device: &Device,
		config: &mut StreamConfig,
		num_stream_errors_discarded: Arc<AtomicU64>,
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
		let (mut unhandled_stream_error_producer, unhandled_stream_error_consumer) =
			RingBuffer::new(64);
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
			move |error| match unhandled_stream_error_producer.push(error) {
				Ok(()) => {}
				Err(PushError::Full(_stream_error)) => {
					num_stream_errors_discarded.fetch_add(1, Ordering::AcqRel);
				}
			},
			None,
		)?;
		stream.play()?;
		self.state = State::Running {
			stream,
			renderer_consumer,
		};
		Ok(unhandled_stream_error_consumer)
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
