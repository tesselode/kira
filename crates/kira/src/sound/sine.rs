mod builder;
mod handle;
mod sound;

pub use builder::*;
pub use handle::*;

use crate::{command::ValueChangeCommand, command_writers_and_readers};

command_writers_and_readers! {
	set_frequency: ValueChangeCommand<f64>,
}
