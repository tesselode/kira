use crate::command::handle_param_setters;

use super::CommandWriters;

pub struct SineHandle {
	pub(super) command_writers: CommandWriters,
}

impl SineHandle {
	handle_param_setters! {
		frequency: f64,
	}
}
