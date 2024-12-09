//! Plays audio using [cpal](https://crates.io/crates/cpal).

#![cfg_attr(docsrs, doc(cfg(feature = "cpal")))]

mod error;
use cpal::{BufferSize, Device};
pub use error::*;

/// Settings for the [`cpal`] backend.
pub struct CpalBackendSettings {
	/// The output audio device to use. If [`None`], the default output
	/// device will be used.
	pub device: Option<Device>,
	/// The buffer size used by the device. If it is set to [`BufferSize::Default`],
	/// the default buffer size for the device will be used. Note that the default
	/// buffer size might be surprisingly large, leading to latency issues. If
	/// a lower latency is desired, consider using [`BufferSize::Fixed`] in accordance
	/// with the [`cpal::SupportedBufferSize`] range provided by the [`cpal::SupportedStreamConfig`]
	/// API.
	pub buffer_size: BufferSize,
}

impl Default for CpalBackendSettings {
	fn default() -> Self {
		Self {
			device: None,
			buffer_size: BufferSize::Default,
		}
	}
}

// TODO: fix WASM backend
#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::CpalBackend;

#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::CpalBackend;
