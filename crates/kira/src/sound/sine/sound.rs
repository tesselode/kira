use std::f64::consts::TAU;

use crate::{
	command::read_commands_into_parameters, info::Info, sound::Sound, Frame, Parameter, StartTime,
	Value,
};

use super::CommandReaders;

pub(super) struct Sine {
	command_readers: CommandReaders,
	frequency: Parameter<f64>,
	start_time: StartTime,
	phase: f64,
}

impl Sine {
	pub(super) fn new(
		command_readers: CommandReaders,
		frequency: Value<f64>,
		start_time: StartTime,
	) -> Self {
		Self {
			command_readers,
			frequency: Parameter::new(frequency, 440.0),
			start_time,
			phase: 0.0,
		}
	}
}

impl Sound for Sine {
	fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, frequency);
	}

	fn process(&mut self, out: &mut [Frame], dt: f64, info: &Info) {
		for (i, frame) in out.iter_mut().enumerate() {
			let single_frame_info = info.for_single_frame(i);
			self.frequency.update(dt, &single_frame_info);
			self.start_time.update(dt, &single_frame_info);
			if self.start_time != StartTime::Immediate {
				*frame = Frame::ZERO;
				continue;
			}
			*frame = Frame::from_mono(0.1 * (self.phase * TAU).sin() as f32);
			self.phase += self.frequency.value() * dt;
			self.phase %= 1.0;
		}
	}

	fn finished(&self) -> bool {
		false
	}
}
