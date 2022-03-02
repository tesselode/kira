//! Plays audio using [cpal](https://crates.io/crates/cpal).

#![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

mod error;
pub use error::*;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::CpalBackend;

#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::CpalBackend;
