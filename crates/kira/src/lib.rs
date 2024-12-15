#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::enum_variant_names)]
#![warn(clippy::todo)]
// #![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

const INTERNAL_BUFFER_SIZE: usize = 128;

mod arena;
pub mod backend;
pub mod clock;
pub mod command;
mod decibels;
pub mod effect;
mod error;
mod frame;
pub mod info;
pub mod manager;
pub mod modulator;
mod panning;
pub mod renderer;
mod resources;
pub mod sound;
mod start_time;
pub mod track;
mod tween;
mod value;

pub use decibels::*;
pub use error::*;
pub use frame::*;
pub use panning::*;
pub use start_time::*;
pub use tween::*;
pub use value::*;
