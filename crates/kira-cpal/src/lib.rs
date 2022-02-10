/*!
# kira-cpal

kira-cpal is a [Kira](https://crates.io/crates/kira) backend
for desktop targets.

## Examples

### Setting up an `AudioManager` with a `CpalBackend`

```no_run
use kira::manager::{AudioManager, AudioManagerSettings};
use kira_cpal::CpalBackend;

let mut manager = AudioManager::new(
	CpalBackend::new()?,
	AudioManagerSettings::default(),
)?;
# Result::<(), Box<dyn std::error::Error>>::Ok(())
```
*/

#![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

use std::{
	error::Error,
	fmt::{Display, Formatter},
	time::Duration,
};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	BuildStreamError, DefaultStreamConfigError, PlayStreamError, Stream, StreamConfig,
};
use kira::manager::backend::{Backend, Renderer};
use ringbuf::{Producer, RingBuffer};

/// An error that can occur when creating a [`CpalBackend`].
#[derive(Debug)]
#[non_exhaustive]
pub enum DeviceSetupError {
	/// A default audio output device could not be determined.
	NoDefaultOutputDevice,
	/// An error occurred when getting the default output configuration.
	DefaultStreamConfigError(DefaultStreamConfigError),
}

impl Display for DeviceSetupError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			DeviceSetupError::NoDefaultOutputDevice => {
				f.write_str("Cannot find the default audio output device")
			}
			DeviceSetupError::DefaultStreamConfigError(error) => error.fmt(f),
		}
	}
}

impl Error for DeviceSetupError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			DeviceSetupError::DefaultStreamConfigError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<DefaultStreamConfigError> for DeviceSetupError {
	fn from(v: DefaultStreamConfigError) -> Self {
		Self::DefaultStreamConfigError(v)
	}
}

/// Errors that can occur when initializing a [`CpalBackend`].
#[derive(Debug)]
#[non_exhaustive]
pub enum InitError {
	/// A default audio output device could not be determined.
	NoDefaultOutputDevice,
	/// An error occured when building the audio stream.
	BuildStreamError(BuildStreamError),
	/// An error occured when starting the audio stream.
	PlayStreamError(PlayStreamError),
}

impl Display for InitError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			InitError::NoDefaultOutputDevice => {
				f.write_str("Cannot find the default audio output device")
			}
			InitError::BuildStreamError(error) => error.fmt(f),
			InitError::PlayStreamError(error) => error.fmt(f),
		}
	}
}

impl Error for InitError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			InitError::BuildStreamError(error) => Some(error),
			InitError::PlayStreamError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<BuildStreamError> for InitError {
	fn from(v: BuildStreamError) -> Self {
		Self::BuildStreamError(v)
	}
}

impl From<PlayStreamError> for InitError {
	fn from(v: PlayStreamError) -> Self {
		Self::PlayStreamError(v)
	}
}

enum State {
	Uninitialized {
		config: StreamConfig,
	},
	Initialized {
		stream_quit_signal_producer: Producer<()>,
	},
}

/// A backend that connects Kira to the operating system's
/// audio APIs using [cpal](https://crates.io/crates/cpal).
pub struct CpalBackend {
	state: State,
}

impl CpalBackend {
	/// Creates a new [`CpalBackend`].
	pub fn new() -> Result<Self, DeviceSetupError> {
		let config = std::thread::spawn(|| -> Result<StreamConfig, DeviceSetupError> {
			let host = cpal::default_host();
			let device = host
				.default_output_device()
				.ok_or(DeviceSetupError::NoDefaultOutputDevice)?;
			Ok(device.default_output_config()?.config())
		})
		.join()
		.unwrap()?;
		Ok(Self {
			state: State::Uninitialized { config },
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

	fn init(&mut self, renderer: Renderer) -> Result<(), Self::InitError> {
		match &mut self.state {
			State::Uninitialized { config } => {
				let config = config.clone();
				let (mut setup_result_producer, mut setup_result_consumer) =
					RingBuffer::<Result<(), Self::InitError>>::new(1).split();
				let (stream_quit_signal_producer, mut stream_quit_signal_consumer) =
					RingBuffer::new(1).split();
				std::thread::spawn(move || {
					// try setting up the stream and send back the result
					let _stream = match setup_stream(config, renderer) {
						Ok(stream) => {
							setup_result_producer.push(Ok(())).unwrap();
							stream
						}
						Err(error) => {
							setup_result_producer.push(Err(error)).unwrap();
							return;
						}
					};
					// sleep until the thread receives the quit signal
					loop {
						std::thread::sleep(Duration::from_millis(100));
						if stream_quit_signal_consumer.pop().is_some() {
							break;
						}
					}
				});
				// wait for a result to come back
				loop {
					if let Some(result) = setup_result_consumer.pop() {
						break result;
					}
					std::thread::sleep(Duration::from_millis(10));
				}?;
				self.state = State::Initialized {
					stream_quit_signal_producer,
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
			stream_quit_signal_producer,
		} = &mut self.state
		{
			if stream_quit_signal_producer.push(()).is_err() {
				panic!("Could not send the quit signal to end the audio thread");
			}
		}
	}
}

fn setup_stream(config: StreamConfig, mut renderer: Renderer) -> Result<Stream, InitError> {
	let device = cpal::default_host()
		.default_output_device()
		.ok_or(InitError::NoDefaultOutputDevice)?;
	let channels = config.channels;
	let stream = device.build_output_stream(
		&config,
		move |data: &mut [f32], _| {
			#[cfg(feature = "assert_no_alloc")]
			assert_no_alloc::assert_no_alloc(|| renderer.on_start_processing());
			#[cfg(not(feature = "assert_no_alloc"))]
			renderer.on_start_processing();
			for frame in data.chunks_exact_mut(channels as usize) {
				#[cfg(feature = "assert_no_alloc")]
				let out = assert_no_alloc::assert_no_alloc(|| renderer.process());
				#[cfg(not(feature = "assert_no_alloc"))]
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
