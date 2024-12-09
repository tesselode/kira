#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::enum_variant_names)]
#![warn(clippy::todo)]
// TODO: turn back on
// #![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

pub const INTERNAL_BUFFER_SIZE: usize = 128;

mod arena;
pub mod backend;
mod error;
mod frame;
pub mod manager;
mod panning;
pub mod renderer;
mod resources;
pub mod sound;

pub use error::*;
pub use frame::*;
pub use panning::*;
