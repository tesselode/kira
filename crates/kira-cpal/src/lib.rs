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

use std::fmt::{Display, Formatter};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	BuildStreamError, DefaultStreamConfigError, Device, PlayStreamError, Stream, StreamConfig,
};
use kira::manager::backend::{Backend, Renderer};

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
	NotStarted,
	Started { _stream: Stream },
}

/// A backend that connects Kira to the operating system's
/// audio APIs using [cpal](https://crates.io/crates/cpal).
pub struct CpalBackend {
	device: Device,
	config: StreamConfig,
	state: State,
}

impl Backend for CpalBackend {
	type Settings = ();

	type Error = Error;

	fn setup(_settings: Self::Settings) -> Result<(Self, u32), Self::Error> {
		let (device, config) = default_device_and_config()?;
		let sample_rate = config.sample_rate.0;
		Ok((
			Self {
				device,
				config,
				state: State::NotStarted,
			},
			sample_rate,
		))
	}

	fn start(&mut self, mut renderer: Renderer) -> Result<(), Self::Error> {
		let channels = self.config.channels;
		let stream = self.device.build_output_stream(
			&self.config,
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
			|_| {},
		)?;
		stream.play()?;
		self.state = State::Started { _stream: stream };
		Ok(())
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
