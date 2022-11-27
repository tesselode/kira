/*!
# Kira

Kira is a backend-agnostic library to create expressive audio
for games. It provides [tweens](tween) for smoothly adjusting
properties of sounds, a flexible [mixer](track) for applying
effects to audio, and a [clock] system for precisely timing
audio events.

The [book](https://tesselode.github.io/kira/) has tutorials
on how to use Kira.
*/

#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::enum_variant_names)]
#![warn(clippy::todo)]
// #![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

pub mod clock;
mod clock_speed;
pub mod dsp;
mod error;
mod loop_behavior;
pub mod manager;
mod output_destination;
mod playback_rate;
pub mod sound;
pub mod spatial;
mod start_time;
pub mod track;
pub mod tween;
mod volume;

pub use clock_speed::*;
pub use error::*;
pub use loop_behavior::*;
pub use output_destination::*;
pub use playback_rate::*;
pub use start_time::*;
pub use volume::*;
