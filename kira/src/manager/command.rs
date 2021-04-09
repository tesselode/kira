use std::sync::Arc;

use crate::sound::Sound;

pub enum Command {
	PlaySound { sound: Arc<Sound> },
}
