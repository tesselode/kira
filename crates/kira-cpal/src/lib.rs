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

use std::{
	fmt::{Display, Formatter},
	time::Duration,
};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	BuildStreamError, DefaultStreamConfigError, PlayStreamError, Stream, StreamConfig,
};
use kira::manager::backend::{Backend, Renderer};
use ringbuf::{Producer, RingBuffer};

/// Errors that can occur when using the cpal backend.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	/// A default audio output device could not be determined.
	NoDefaultOutputDevice,
	/// An error occurred when getting the default output configuration.
	DefaultStreamConfigError(DefaultStreamConfigError),
	/// An error occured when building the audio stream.
	BuildStreamError(BuildStreamError),
	/// An error occured when starting the audio stream.
	PlayStreamError(PlayStreamError),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::NoDefaultOutputDevice => {
				f.write_str("Cannot find the default audio output device")
			}
			Error::DefaultStreamConfigError(error) => error.fmt(f),
			Error::BuildStreamError(error) => error.fmt(f),
			Error::PlayStreamError(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Error::DefaultStreamConfigError(error) => Some(error),
			Error::BuildStreamError(error) => Some(error),
			Error::PlayStreamError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<DefaultStreamConfigError> for Error {
	fn from(v: DefaultStreamConfigError) -> Self {
		Self::DefaultStreamConfigError(v)
	}
}

impl From<BuildStreamError> for Error {
	fn from(v: BuildStreamError) -> Self {
		Self::BuildStreamError(v)
	}
}

impl From<PlayStreamError> for Error {
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

impl Backend for CpalBackend {
	type Settings = ();

	type Error = Error;

	fn setup(_settings: Self::Settings) -> Result<Self, Self::Error> {
		let config = std::thread::spawn(|| -> Result<StreamConfig, Error> {
			let host = cpal::default_host();
			let device = host
				.default_output_device()
				.ok_or(Error::NoDefaultOutputDevice)?;
			Ok(device.default_output_config()?.config())
		})
		.join()
		.unwrap()?;
		Ok(Self {
			state: State::Uninitialized { config },
		})
	}

	fn sample_rate(&self) -> u32 {
		match &self.state {
			State::Uninitialized { config, .. } => config.sample_rate.0,
			State::Initialized { .. } => unreachable!(),
		}
	}

	fn start(&mut self, renderer: Renderer) -> Result<(), Self::Error> {
		match &mut self.state {
			State::Uninitialized { config } => {
				let config = config.clone();
				let (mut setup_result_producer, mut setup_result_consumer) =
					RingBuffer::<Result<(), Self::Error>>::new(1).split();
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

fn setup_stream(config: StreamConfig, mut renderer: Renderer) -> Result<Stream, Error> {
	let device = cpal::default_host()
		.default_output_device()
		.ok_or(Error::NoDefaultOutputDevice)?;
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
