mod stream_manager;

use stream_manager::{StreamManager, StreamManagerController};

use crate::manager::backend::{Backend, Renderer};
use cpal::{
	traits::{DeviceTrait, HostTrait},
	Device, StreamConfig,
};

use super::Error;

enum State {
	Empty,
	Uninitialized {
		device: Device,
		config: StreamConfig,
		follow_device: bool,
	},
	Initialized {
		stream_manager_controller: StreamManagerController,
	},
}

pub struct CpalSettings {
	// Whether the default output device should be polled for changes to the output configuration
	// and the stream restarted if needed. This defaults to false on macOS do to known issues with
	// audio artifacts when querying the default output device while it is producing sound.
	// The stream will still restart if the device returns a disconnected error.
	pub follow_default_output_device: bool,
}

impl Default for CpalSettings {
	fn default() -> Self {
		Self {
			follow_default_output_device: cfg!(not(target_os = "macos")),
		}
	}
}

/// A backend that uses [cpal](https://crates.io/crates/cpal) to
/// connect a [`Renderer`] to the operating system's audio driver.
pub struct CpalBackend {
	state: State,
}

impl Backend for CpalBackend {
	type Settings = CpalSettings;

	type Error = Error;

	fn setup(settings: Self::Settings) -> Result<(Self, u32), Self::Error> {
		let host = cpal::default_host();
		let device = host
			.default_output_device()
			.ok_or(Error::NoDefaultOutputDevice)?;
		let config = device.default_output_config()?.config();
		let sample_rate = config.sample_rate.0;
		Ok((
			Self {
				state: State::Uninitialized {
					device,
					config,
					follow_device: settings.follow_default_output_device,
				},
			},
			sample_rate,
		))
	}

	fn start(&mut self, renderer: Renderer) -> Result<(), Self::Error> {
		let state = std::mem::replace(&mut self.state, State::Empty);
		if let State::Uninitialized {
			device,
			config,
			follow_device,
		} = state
		{
			self.state = State::Initialized {
				stream_manager_controller: StreamManager::start(
					renderer,
					device,
					config,
					follow_device,
				),
			};
		} else {
			panic!("Cannot initialize the backend multiple times")
		}
		Ok(())
	}
}

impl Drop for CpalBackend {
	fn drop(&mut self) {
		if let State::Initialized {
			stream_manager_controller,
		} = &self.state
		{
			stream_manager_controller.stop();
		}
	}
}
