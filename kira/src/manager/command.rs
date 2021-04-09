use std::sync::Arc;

use ringbuf::Consumer;

use crate::sound::{instance, Sound};

pub(crate) enum Command {
	PlaySound {
		sound: Arc<Sound>,
		command_consumer: Consumer<instance::Command>,
	},
}
