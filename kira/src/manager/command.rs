use ringbuf::Consumer;

use crate::sound::instance::{self, Instance};

pub(crate) enum Command {
	PlaySound { instance: Instance },
}
