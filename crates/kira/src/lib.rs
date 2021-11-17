//! # Kira

#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::large_enum_variant)]
#![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

pub mod clock;
pub mod dsp;
mod error;
mod loop_behavior;
pub mod manager;
pub mod parameter;
pub mod sound;
mod start_time;
pub mod track;
pub mod tween;
pub mod value;

pub use error::*;
pub use loop_behavior::*;
pub use start_time::*;
