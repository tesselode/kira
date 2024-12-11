#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::enum_variant_names)]
#![warn(clippy::todo)]
// #![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

pub mod backend;
mod frame;
pub mod manager;
mod panning;
pub mod renderer;

pub use frame::*;
pub use panning::*;
