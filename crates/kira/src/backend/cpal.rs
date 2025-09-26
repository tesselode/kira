//! Plays audio using [cpal](https://crates.io/crates/cpal).

#![cfg_attr(docsrs, doc(cfg(feature = "cpal")))]

mod error;
use cpal::{Device, StreamConfig};
pub use error::*;

/// Settings for the cpal backend.
#[derive(Clone, Default)]
pub struct CpalBackendSettings {
	/// The output audio device to use. If [`None`], the default output
	/// device will be used.
	pub device: Option<Device>,
	/// A StreamConfig given by Cpal. If [`None`], the default supported
	/// config will be used. You can also get a supported config of your
	/// choosing using Cpal functions.
	pub config: Option<StreamConfig>,
}

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::CpalBackend;

#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::CpalBackend;
