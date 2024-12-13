use crate::{
	sound::{Sound, SoundData},
	StartTime, Value,
};

use super::{command_writers_and_readers, sound::Sine, SineHandle};

pub struct SineBuilder {
	pub frequency: Value<f64>,
	pub start_time: StartTime,
}

impl Default for SineBuilder {
	fn default() -> Self {
		Self {
			frequency: Value::Fixed(440.0),
			start_time: StartTime::Immediate,
		}
	}
}

impl SoundData for SineBuilder {
	type Error = ();

	type Handle = SineHandle;

	fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
		let (command_writers, command_readers) = command_writers_and_readers();
		let sound = Box::new(Sine::new(command_readers, self.frequency, self.start_time));
		let handle = SineHandle { command_writers };
		Ok((sound, handle))
	}
}
