mod emitter;
mod handle;
mod settings;
mod sound;

pub use emitter::*;
pub use handle::*;
pub use settings::*;

use crate::{command::ValueChangeCommand, command_writers_and_readers};

use glam::{Quat, Vec3};

command_writers_and_readers! {
	set_listener_position: ValueChangeCommand<Vec3>,
	set_listener_orientation: ValueChangeCommand<Quat>,
}
